//! A reader for wasmquay's WIT-like interface manifest.
//!
//! The WebAssembly component model describes interfaces in WIT (Wasm Interface
//! Types). Rather than embed a full WIT parser, wasmquay defines a small,
//! well-specified textual subset that captures the pieces relevant to
//! capability analysis: the interfaces a component *imports* (its host
//! requirements) and the ones it *exports* (its public surface). The grammar
//! is documented in `docs/FORMAT.md`.
//!
//! Example manifest:
//!
//! ```text
//! package acme:image-tool@1.2.0
//!
//! world processor {
//!   import wasi:filesystem/types
//!   import wasi:clocks/wall-clock
//!   export process: func(input: list<u8>) -> list<u8>
//! }
//! ```
//!
//! The reader is line oriented, tolerant of blank lines and `//` comments, and
//! never panics on malformed input.

use crate::error::{Error, ErrorKind, Result};

/// One `import` or `export` declaration in a world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceRef {
    /// The interface path, e.g. `wasi:filesystem/types`.
    pub path: String,
    /// The signature text for a `func` export, if present (after `:`).
    pub signature: Option<String>,
}

/// A `world` block: a named set of imports and exports.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct World {
    /// The world's name.
    pub name: String,
    /// Interfaces the world imports (its host requirements).
    pub imports: Vec<InterfaceRef>,
    /// Interfaces the world exports (its public surface).
    pub exports: Vec<InterfaceRef>,
}

/// A parsed manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Manifest {
    /// The `package name:pkg@version` line, if present.
    pub package: Option<String>,
    /// The worlds declared in the manifest.
    pub worlds: Vec<World>,
}

impl Manifest {
    /// All imported interface paths across every world, de-duplicated & sorted.
    pub fn all_imports(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .worlds
            .iter()
            .flat_map(|w| w.imports.iter().map(|i| i.path.clone()))
            .collect();
        v.sort();
        v.dedup();
        v
    }

    /// All exported interface paths across every world, de-duplicated & sorted.
    pub fn all_exports(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .worlds
            .iter()
            .flat_map(|w| w.exports.iter().map(|i| i.path.clone()))
            .collect();
        v.sort();
        v.dedup();
        v
    }
}

/// Parse a manifest from its textual form.
pub fn parse(text: &str) -> Result<Manifest> {
    let mut manifest = Manifest::default();
    let mut current: Option<World> = None;

    for (lineno, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("package ") {
            if current.is_some() {
                return Err(Error::new(
                    ErrorKind::BadManifest,
                    format!(
                        "line {}: 'package' cannot appear inside a world",
                        lineno + 1
                    ),
                ));
            }
            manifest.package = Some(rest.trim().to_string());
            continue;
        }

        if let Some(rest) = line.strip_prefix("world ") {
            if let Some(w) = current.take() {
                manifest.worlds.push(w);
            }
            let name = rest.trim_end_matches('{').trim().to_string();
            if name.is_empty() {
                return Err(Error::new(
                    ErrorKind::BadManifest,
                    format!("line {}: world requires a name", lineno + 1),
                ));
            }
            current = Some(World {
                name,
                ..World::default()
            });
            continue;
        }

        if line == "}" {
            match current.take() {
                Some(w) => manifest.worlds.push(w),
                None => {
                    return Err(Error::new(
                        ErrorKind::BadManifest,
                        format!("line {}: unmatched '}}'", lineno + 1),
                    ))
                }
            }
            continue;
        }

        if let Some(rest) = line.strip_prefix("import ") {
            let iref = parse_interface_ref(rest)?;
            push_into(&mut current, lineno, |w| w.imports.push(iref))?;
            continue;
        }

        if let Some(rest) = line.strip_prefix("export ") {
            let iref = parse_interface_ref(rest)?;
            push_into(&mut current, lineno, |w| w.exports.push(iref))?;
            continue;
        }

        // Opening brace on its own line for `world name` split across lines.
        if line == "{" && current.is_some() {
            continue;
        }

        return Err(Error::new(
            ErrorKind::BadManifest,
            format!("line {}: unrecognized directive '{}'", lineno + 1, line),
        ));
    }

    if let Some(w) = current.take() {
        // A missing closing brace is tolerated; treat EOF as end of world.
        manifest.worlds.push(w);
    }

    Ok(manifest)
}

fn push_into<F: FnOnce(&mut World)>(
    current: &mut Option<World>,
    lineno: usize,
    f: F,
) -> Result<()> {
    match current.as_mut() {
        Some(w) => {
            f(w);
            Ok(())
        }
        None => Err(Error::new(
            ErrorKind::BadManifest,
            format!("line {}: import/export outside of a world", lineno + 1),
        )),
    }
}

fn parse_interface_ref(rest: &str) -> Result<InterfaceRef> {
    let rest = rest.trim().trim_end_matches('{').trim();
    // A signature is introduced by ": " (colon followed by whitespace), e.g.
    // `export process: func(...)`. Interface paths such as
    // `wasi:filesystem/types` use a colon with no trailing space, so we must
    // not split on those. We therefore look for the first ": " boundary.
    if let Some(idx) = find_signature_boundary(rest) {
        let path = rest[..idx].trim();
        let sig = rest[idx + 1..].trim();
        Ok(InterfaceRef {
            path: path.to_string(),
            signature: Some(sig.to_string()),
        })
    } else {
        Ok(InterfaceRef {
            path: rest.to_string(),
            signature: None,
        })
    }
}

/// Find the byte index of a `:` that is immediately followed by whitespace.
fn find_signature_boundary(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b':' {
            match bytes.get(i + 1) {
                Some(next) if next.is_ascii_whitespace() => return Some(i),
                None => return Some(i),
                _ => {}
            }
        }
    }
    None
}

fn strip_comment(line: &str) -> &str {
    match line.find("//") {
        Some(idx) => &line[..idx],
        None => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
        package acme:image@1.0.0
        // a comment
        world processor {
            import wasi:filesystem/types
            import wasi:clocks/wall-clock
            export process: func(input: list<u8>) -> list<u8>
        }
    "#;

    #[test]
    fn parses_package_and_world() {
        let m = parse(SAMPLE).unwrap();
        assert_eq!(m.package.as_deref(), Some("acme:image@1.0.0"));
        assert_eq!(m.worlds.len(), 1);
        assert_eq!(m.worlds[0].name, "processor");
        assert_eq!(m.worlds[0].imports.len(), 2);
        assert_eq!(m.worlds[0].exports.len(), 1);
    }

    #[test]
    fn export_signature_captured() {
        let m = parse(SAMPLE).unwrap();
        let exp = &m.worlds[0].exports[0];
        assert_eq!(exp.path, "process");
        assert_eq!(
            exp.signature.as_deref(),
            Some("func(input: list<u8>) -> list<u8>")
        );
    }

    #[test]
    fn aggregates_imports_sorted() {
        let m = parse(SAMPLE).unwrap();
        assert_eq!(
            m.all_imports(),
            vec![
                "wasi:clocks/wall-clock".to_string(),
                "wasi:filesystem/types".to_string()
            ]
        );
    }

    #[test]
    fn rejects_directive_outside_world() {
        let err = parse("import wasi:x/y").unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BadManifest);
    }

    #[test]
    fn tolerates_missing_close_brace() {
        let m = parse("world w {\n  import a:b/c").unwrap();
        assert_eq!(m.worlds.len(), 1);
        assert_eq!(m.worlds[0].imports.len(), 1);
    }
}
