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
