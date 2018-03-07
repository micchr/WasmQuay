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
