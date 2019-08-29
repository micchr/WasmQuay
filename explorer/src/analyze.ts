/**
 * Static capability analysis over parsed wasmquay reports.
 *
 * The explorer's job is to turn a raw inspection (and optionally a policy
 * report) into an opinionated, human-facing risk view. Nothing here executes
 * WebAssembly — it is a pure function of the JSON the Rust CLI produced.
 */

import type {
  Domain,
  InspectionReport,
  PolicyReport,
} from "./types.js";
import { ALL_DOMAINS } from "./types.js";

/** A coarse risk weight per capability domain (0 = inert, 5 = highest). */
export const DOMAIN_WEIGHT: Readonly<Record<Domain, number>> = {
  network: 5,
  fs: 4,
  unknown: 5,
  env: 2,
  random: 1,
  clock: 1,
  stdio: 1,
};

/** A short rationale shown alongside each domain in explanations. */
export const DOMAIN_RATIONALE: Readonly<Record<Domain, string>> = {
  network: "can open sockets / exfiltrate data",
  fs: "can read or modify files",
  unknown: "imports an unrecognized host — treat as untrusted",
  env: "can read environment variables and arguments",
  random: "consumes a randomness source",
  clock: "can read wall/monotonic clocks",
  stdio: "uses standard input/output streams",
};

/** The risk band a component falls into. */
export type RiskBand = "inert" | "low" | "moderate" | "elevated" | "high";

/** The result of analyzing a single inspection report. */
export interface CapabilitySurface {
  readonly source: string;
  readonly moduleName: string | null;
