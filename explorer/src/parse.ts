/**
 * Runtime validation for wasmquay reports.
 *
 * `JSON.parse` yields `unknown`; these guards narrow it to the typed shapes in
 * `types.ts` and throw a descriptive `ReportError` when a document does not
 * match its declared schema. This keeps the rest of the explorer honest about
 * what it is consuming.
 */

import type {
  CompatReport,
  Domain,
  InspectionReport,
  PolicyReport,
} from "./types.js";
import { ALL_DOMAINS } from "./types.js";

/** Thrown when a report does not conform to its expected schema. */
export class ReportError extends Error {
  constructor(message: string) {
