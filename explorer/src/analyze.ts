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
