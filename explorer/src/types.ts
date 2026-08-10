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
  readonly custom_name: string | null;
  readonly offset: number;
  readonly size: number;
}

/** A single classified capability requirement. */
export interface CapabilityRequirement {
  readonly source: string;
  readonly detail: string;
}

/** Requirements grouped by domain. */
export interface CapabilityGroup {
  readonly domain: Domain;
  readonly requirements: readonly CapabilityRequirement[];
}

/** The `wasmquay/inspection@1` document. */
export interface InspectionReport {
  readonly schema: string;
  readonly source: string;
  readonly header: {
    readonly magic: string;
    readonly version: number;
    readonly byte_length: number;
    readonly module_name: string | null;
  };
  readonly sections: readonly SectionEntry[];
  readonly imports: readonly ImportEntry[];
  readonly exports: readonly ExportEntry[];
  readonly names: readonly { readonly index: number; readonly name: string }[];
  readonly capabilities: readonly CapabilityGroup[];
}

/** A single per-domain verdict from a policy report. */
export interface PolicyVerdict {
  readonly domain: Domain;
  readonly required: boolean;
  readonly allowed: boolean;
  readonly violation: boolean;
  readonly requirements: readonly string[];
}

/** The `wasmquay/policy@1` document. */
export interface PolicyReport {
  readonly schema: string;
  readonly policy: string;
  readonly compliant: boolean;
  readonly violations: readonly Domain[];
  readonly verdicts: readonly PolicyVerdict[];
}

/** The `wasmquay/compat@1` document. */
export interface CompatReport {
  readonly schema: string;
  readonly baseline: string;
  readonly candidate: string;
  readonly compatible: boolean;
  readonly breaking_reasons: readonly string[];
  readonly export_diffs: readonly {
    readonly change: "added" | "removed";
    readonly name: string;
    readonly kind: string;
  }[];
  readonly capability_diffs: readonly {
    readonly change: "added" | "removed";
    readonly domain: Domain;
    readonly source: string;
  }[];
}

/** All known capability domains in a stable order. */
export const ALL_DOMAINS: readonly Domain[] = [
  "fs",
  "env",
  "clock",
  "network",
  "random",
  "stdio",
  "unknown",
];
