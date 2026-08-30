//! `wasmquay` — the command-line front end.
//!
//! Subcommands:
//!
//! * `inspect <file.wasm>`      — decode and summarize a module.
//! * `caps <file.wasm>`         — list the classified capability requirements.
//! * `policy <file.wasm> <pol>` — evaluate a module against a policy file.
//! * `compat <base> <cand>`     — compare two modules for substitutability.
//! * `manifest <file.wit>`      — parse and summarize a WIT-like manifest.
//! * `gen-fixtures <dir>`       — write the bundled binary fixtures to a dir.
//!
//! Global flags: `--json` (machine output), `--pretty` (pretty JSON),
//! `--help`, `--version`.
//!
//! Exit codes: `0` success/compliant, `1` usage error, `2` parse/IO error,
//! `3` policy violation or incompatibility (so CI can gate on it).

use std::path::Path;
use std::process::ExitCode;

use wasmquay_core::{compat, fixture::FixtureBuilder, policy, report, wasm, wit, VERSION};

const USAGE: &str = "\
wasmquay — offline WebAssembly component inspection & capability policy

USAGE:
    wasmquay <COMMAND> [ARGS] [--json] [--pretty]

COMMANDS:
    inspect <file.wasm>                 Decode header/sections/imports/exports
    caps <file.wasm>                    List classified capability requirements
    policy <file.wasm> <policy.pol>     Evaluate module against a policy
    compat <baseline.wasm> <cand.wasm>  Compare two modules for compatibility
    manifest <file.wit>                 Parse a WIT-like interface manifest
    gen-fixtures <dir>                  Write bundled .wasm fixtures to <dir>

FLAGS:
    --json      Emit machine-readable JSON
    --pretty    Pretty-print JSON (implies --json)
    -h, --help  Show this help
    -V, --version  Show version

EXIT CODES:
    0 ok / compliant / compatible
    1 usage error
    2 parse or I/O error
    3 policy violation or incompatibility
";

/// Output format selected by global flags.
#[derive(Clone, Copy, PartialEq)]
enum Format {
    Text,
    Json,
    PrettyJson,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {}", e);
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{}", USAGE);
        return Ok(ExitCode::SUCCESS);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("wasmquay {}", VERSION);
        return Ok(ExitCode::SUCCESS);
    }

    let mut format = Format::Text;
    let mut positional: Vec<String> = Vec::new();
    for a in args {
        match a.as_str() {
            "--json" => {
                if format == Format::Text {
                    format = Format::Json;
                }
            }
            "--pretty" => format = Format::PrettyJson,
            flag if flag.starts_with('-') => {
                return Err(format!("unknown flag '{}'\n\n{}", flag, USAGE));
            }
            other => positional.push(other.to_string()),
        }
    }

    let command = positional.first().cloned().unwrap_or_default();
    let rest = &positional[1.min(positional.len())..];

    match command.as_str() {
        "inspect" => cmd_inspect(rest, format),
        "caps" => cmd_caps(rest, format),
        "policy" => cmd_policy(rest, format),
        "compat" => cmd_compat(rest, format),
        "manifest" => cmd_manifest(rest, format),
        "gen-fixtures" => cmd_gen_fixtures(rest),
        other => Err(format!("unknown command '{}'\n\n{}", other, USAGE)),
    }
}

fn read_file(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("cannot read '{}': {}", path, e))
}

fn read_text(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read '{}': {}", path, e))
}

fn label_of(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn emit_json(json: wasmquay_core::json::Json, format: Format) {
    match format {
        Format::PrettyJson => println!("{}", json.to_pretty()),
        _ => println!("{}", json.to_compact()),
    }
}

fn cmd_inspect(rest: &[String], format: Format) -> Result<ExitCode, String> {
    let path = rest.first().ok_or_else(|| usage("inspect <file.wasm>"))?;
    let bytes = read_file(path)?;
    let module = wasm::parse(&bytes).map_err(|e| e.to_string())?;
    let reqs = policy::requirements_from_module(&module);
    let label = label_of(path);
    match format {
        Format::Text => print!("{}", report::inspection_text(&module, &label, &reqs)),
        _ => emit_json(report::inspection_json(&module, &label, &reqs), format),
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_caps(rest: &[String], format: Format) -> Result<ExitCode, String> {
    let path = rest.first().ok_or_else(|| usage("caps <file.wasm>"))?;
    let bytes = read_file(path)?;
    let module = wasm::parse(&bytes).map_err(|e| e.to_string())?;
    let reqs = policy::requirements_from_module(&module);
    match format {
        Format::Text => {
            if reqs.is_empty() {
                println!("(no host imports — module is self-contained)");
            }
            for r in &reqs {
                println!("{:<8} {} :: {}", r.domain.slug(), r.source, r.detail);
            }
        }
        _ => emit_json(
            report::inspection_json(&module, &label_of(path), &reqs),
            format,
        ),
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_policy(rest: &[String], format: Format) -> Result<ExitCode, String> {
    let wasm_path = rest
        .first()
        .ok_or_else(|| usage("policy <file.wasm> <policy.pol>"))?;
    let pol_path = rest
        .get(1)
        .ok_or_else(|| usage("policy <file.wasm> <policy.pol>"))?;
    let bytes = read_file(wasm_path)?;
    let module = wasm::parse(&bytes).map_err(|e| e.to_string())?;
    let reqs = policy::requirements_from_module(&module);
    let pol = policy::Policy::parse(&read_text(pol_path)?).map_err(|e| e.to_string())?;
    let eval = policy::evaluate(&reqs, &pol);
    match format {
        Format::Text => print!("{}", report::evaluation_text(&eval)),
        _ => emit_json(report::evaluation_json(&eval), format),
    }
    if eval.is_compliant() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(3))
    }
}

fn cmd_compat(rest: &[String], format: Format) -> Result<ExitCode, String> {
    let base_path = rest
        .first()
        .ok_or_else(|| usage("compat <baseline.wasm> <cand.wasm>"))?;
    let cand_path = rest
        .get(1)
        .ok_or_else(|| usage("compat <baseline.wasm> <cand.wasm>"))?;
    let base = wasm::parse(&read_file(base_path)?).map_err(|e| e.to_string())?;
    let cand = wasm::parse(&read_file(cand_path)?).map_err(|e| e.to_string())?;
    let comp = compat::compare_modules(&base, &label_of(base_path), &cand, &label_of(cand_path));
    match format {
        Format::Text => print!("{}", report::compatibility_text(&comp)),
        _ => emit_json(report::compatibility_json(&comp), format),
    }
    if comp.is_compatible() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(3))
    }
}

fn cmd_manifest(rest: &[String], format: Format) -> Result<ExitCode, String> {
    let path = rest.first().ok_or_else(|| usage("manifest <file.wit>"))?;
    let text = read_text(path)?;
    let manifest = wit::parse(&text).map_err(|e| e.to_string())?;
    let reqs = policy::requirements_from_manifest(&manifest);
    match format {
        Format::Text => {
            if let Some(pkg) = &manifest.package {
                println!("package: {}", pkg);
            }
            for world in &manifest.worlds {
                println!(
                    "world {} ({} imports, {} exports)",
                    world.name,
                    world.imports.len(),
                    world.exports.len()
                );
            }
            println!("classified capability domains:");
            for r in &reqs {
                println!("  {:<8} {}", r.domain.slug(), r.source);
            }
        }
        _ => {
            use wasmquay_core::json::Json;
            let worlds = manifest
                .worlds
                .iter()
                .map(|w| {
                    Json::obj(vec![
                        ("name", Json::s(w.name.clone())),
                        (
                            "imports",
                            Json::str_arr(w.imports.iter().map(|i| i.path.clone())),
                        ),
                        (
                            "exports",
                            Json::str_arr(w.exports.iter().map(|i| i.path.clone())),
                        ),
                    ])
                })
                .collect();
            let caps = reqs
                .iter()
                .map(|r| {
                    Json::obj(vec![
                        ("domain", Json::s(r.domain.slug())),
                        ("source", Json::s(r.source.clone())),
                    ])
                })
                .collect();
            let doc = Json::obj(vec![
                ("schema", Json::s("wasmquay/manifest@1")),
                (
                    "package",
                    match &manifest.package {
                        Some(p) => Json::s(p.clone()),
                        None => Json::Null,
                    },
                ),
                ("worlds", Json::Arr(worlds)),
                ("capabilities", Json::Arr(caps)),
            ]);
            emit_json(doc, format);
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Write the bundled deterministic fixtures. These are the exact binaries the
/// examples and tests reference, generated from the encoder so the repository
/// need not commit opaque blobs (though it also ships copies under `fixtures/`).
fn cmd_gen_fixtures(rest: &[String]) -> Result<ExitCode, String> {
    let dir = rest.first().ok_or_else(|| usage("gen-fixtures <dir>"))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create '{}': {}", dir, e))?;

    for (name, bytes) in bundled_fixtures() {
        let path = Path::new(dir).join(name);
        std::fs::write(&path, &bytes)
            .map_err(|e| format!("cannot write '{}': {}", path.display(), e))?;
        println!("wrote {} ({} bytes)", path.display(), bytes.len());
    }
    Ok(ExitCode::SUCCESS)
}

/// The canonical set of fixtures shared by the CLI, tests and examples.
pub fn bundled_fixtures() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        (
            // A well-behaved service: clock + stdio only.
            "clock-service.wasm",
            FixtureBuilder::new()
                .module_name("clock-service")
                .import_func("wasi_snapshot_preview1", "clock_time_get")
                .import_func("wasi_snapshot_preview1", "fd_write")
                .import_memory("env", "memory")
                .export_func("run", 0)
                .function_name(0, "run")
                .build(),
        ),
        (
            // A networked variant: adds sockets on top of the clock service.
            "net-service.wasm",
            FixtureBuilder::new()
                .module_name("net-service")
                .import_func("wasi_snapshot_preview1", "clock_time_get")
                .import_func("wasi_snapshot_preview1", "fd_write")
                .import_func("wasi_snapshot_preview1", "sock_recv")
                .import_func("wasi_snapshot_preview1", "sock_send")
                .import_memory("env", "memory")
                .export_func("run", 0)
                .function_name(0, "run")
                .build(),
        ),
        (
            // A component-model style module importing preview2 interfaces.
            "fs-component.wasm",
            FixtureBuilder::new()
                .module_name("fs-component")
                .import_func("wasi:filesystem/types", "read-via-stream")
                .import_func("wasi:clocks/wall-clock", "now")
                .export_func("process", 0)
                .export_func("flush", 1)
                .function_name(0, "process")
                .build(),
        ),
        (
            // A module that imports an unrecognized host — flagged as unknown.
            "opaque-host.wasm",
            FixtureBuilder::new()
                .module_name("opaque-host")
                .import_func("mystery_host", "do_the_thing")
                .export_func("run", 0)
                .build(),
        ),
    ]
}

fn usage(spec: &str) -> String {
    format!("usage: wasmquay {}", spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_fixtures_parse() {
        for (name, bytes) in bundled_fixtures() {
            let m = wasm::parse(&bytes).unwrap_or_else(|e| panic!("{} failed: {}", name, e));
            assert_eq!(m.version, 1, "{}", name);
        }
    }

    #[test]
    fn net_service_requires_network() {
        let bytes = &bundled_fixtures()[1].1;
        let m = wasm::parse(bytes).unwrap();
        let reqs = policy::requirements_from_module(&m);
        assert!(reqs.iter().any(|r| r.domain == policy::Domain::Network));
    }

    #[test]
    fn opaque_host_is_unknown() {
        let bytes = &bundled_fixtures()[3].1;
        let m = wasm::parse(bytes).unwrap();
        let reqs = policy::requirements_from_module(&m);
        assert!(reqs.iter().any(|r| r.domain == policy::Domain::Unknown));
    }

    #[test]
    fn label_strips_directory() {
        assert_eq!(label_of("a/b/c.wasm"), "c.wasm");
    }
}
