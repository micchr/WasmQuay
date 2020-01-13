/**
 * Type definitions for the JSON reports emitted by the `wasmquay` CLI.
 *
 * These mirror the schemas documented in `docs/FORMAT.md`. The explorer never
 * mutates a report; it only reads these shapes, so every field is `readonly`.
 */

/** The stable capability domain slugs shared with the Rust core. */
export type Domain =
  | "fs"
  | "env"
  | "clock"
  | "network"
  | "random"
  | "stdio"
  | "unknown";

/** A single import entry from an inspection report. */
export interface ImportEntry {
  readonly module: string;
  readonly field: string;
  readonly kind: string;
}

/** A single export entry from an inspection report. */
export interface ExportEntry {
  readonly field: string;
  readonly kind: string;
  readonly index: number;
}

/** A decoded section header. */
export interface SectionEntry {
  readonly id: number;
  readonly name: string;
