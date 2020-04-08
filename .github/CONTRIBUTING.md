# Contributing to wasmquay

Thanks for considering a contribution. wasmquay is a static analyzer: it reads
WebAssembly bytes and reports the host capabilities a module imports; it never
executes or instantiates a module.

## Development setup

- **Rust (stable, >= 1.74)** - the workspace lives in `crates/`:

  ```sh
  cargo build
  cargo test
  cargo fmt --all -- --check
  cargo clippy --all-targets -- -D warnings
  ```

- **Node 20+** - the TypeScript explorer lives in `explorer/`:

  ```sh
  cd explorer
  npm install
  npm run typecheck
  npm run build
  npm test
  ```

## Before you open a pull request
