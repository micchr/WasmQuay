/**
 * Type definitions for the JSON reports emitted by the `wasmquay` CLI.
 *
 * These mirror the schemas documented in `docs/FORMAT.md`. The explorer never
 * mutates a report; it only reads these shapes, so every field is `readonly`.
