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
