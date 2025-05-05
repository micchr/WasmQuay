# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-08-31

### Added

- **`wasmquay-core`** — zero-dependency Rust library:
  - LEB128 / byte reader (`leb`) with unsigned & signed variable-length ints.
  - WebAssembly core binary decoder (`wasm`): header, section table, import,
    export and custom `name` section decoding. Total (never panics on bad
    input).
  - WIT-like manifest reader (`wit`) for a documented interface subset.
  - Capability classifier and explicit policy evaluator (`policy`) covering the
    `fs`, `env`, `clock`, `network`, `random`, `stdio` and `unknown` domains,
    with deny-by-default and per-domain resource allow-lists.
  - Substitutability comparison (`compat`) between two components.
  - Stable, versioned JSON/text report serialization (`report`) with a
    hand-written ordered JSON emitter (`json`).
  - A matching binary *encoder* (`fixture`) for deterministic test fixtures.
- **`wasmquay` CLI** — `inspect`, `caps`, `policy`, `compat`, `manifest` and
  `gen-fixtures` subcommands with `--json`/`--pretty` output and CI-friendly
  exit codes.
- **`wasmquay-explorer`** — TypeScript library + `wasmquay-explore` CLI that
  consumes the JSON reports and produces capability surfaces, weighted risk
  scores/bands and policy reconciliation findings. Compiles offline with a
  stock `tsc` (no `@types/node` required).
- Binary fixtures (generated) and example policy/manifest files.
- Documentation: `docs/FORMAT.md` (normative formats), two original animated
  SVGs, README, LICENSE and CI workflow.
- Test suites: 43 Rust tests (unit + integration + doctest) and 12 TypeScript
  tests, all passing.

### Notes

- wasmquay performs **static inspection and policy analysis only**. It does not
  execute or instantiate WebAssembly modules and makes no runtime enforcement
  claims. See the README "Scope & honesty" section.

[Unreleased]: https://github.com/micchr/wasmquay/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/micchr/wasmquay/releases/tag/v0.1.0

<!-- draft note 845 -->
