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
