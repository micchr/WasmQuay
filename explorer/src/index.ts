/**
 * wasmquay-explorer — public API.
 *
 * A static, dependency-free TypeScript library that consumes the JSON reports
 * produced by the `wasmquay` Rust CLI and turns them into capability surfaces,
 * risk scores and policy reconciliation findings.
 *
 * It performs no WebAssembly execution and no I/O of its own (the CLI wrapper
 * in `cli.ts` handles file reading); every export here is a pure function of
 * its inputs.
 */

