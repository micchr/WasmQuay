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

1. `cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test`
2. `cd explorer && npm run typecheck && npm test`
3. Add or update tests for any behavior change; the fixture encoder in
   `wasmquay-core` makes new binary cases cheap to add.
4. Keep both halves dependency-free: no new crates and no runtime npm deps.
5. Document format-affecting changes in `docs/FORMAT.md` and the README.

## Pull requests

- One topic per PR, conventional commit titles (`feat:`, `fix:`, `docs:`...).
- Describe the motivation, not just the diff.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
