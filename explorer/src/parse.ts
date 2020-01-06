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
    super(message);
    this.name = "ReportError";
  }
}

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function requireString(obj: Record<string, unknown>, key: string): string {
  const v = obj[key];
  if (typeof v !== "string") {
    throw new ReportError(`expected string field '${key}'`);
  }
  return v;
}

function requireNumber(obj: Record<string, unknown>, key: string): number {
  const v = obj[key];
  if (typeof v !== "number") {
    throw new ReportError(`expected number field '${key}'`);
  }
  return v;
}

function requireBool(obj: Record<string, unknown>, key: string): boolean {
  const v = obj[key];
  if (typeof v !== "boolean") {
    throw new ReportError(`expected boolean field '${key}'`);
  }
  return v;
}

function requireArray(obj: Record<string, unknown>, key: string): unknown[] {
  const v = obj[key];
  if (!Array.isArray(v)) {
    throw new ReportError(`expected array field '${key}'`);
  }
  return v;
}

function asDomain(v: unknown): Domain {
  if (typeof v === "string" && (ALL_DOMAINS as readonly string[]).includes(v)) {
    return v as Domain;
  }
  throw new ReportError(`unknown capability domain '${String(v)}'`);
}

/** Parse and validate an inspection report from a JSON string. */
export function parseInspection(text: string): InspectionReport {
  const doc = parseJson(text);
  if (!doc.schema || !String(doc.schema).startsWith("wasmquay/inspection")) {
    throw new ReportError(`not an inspection report (schema='${String(doc.schema)}')`);
  }
  const header = doc.header;
  if (!isObject(header)) {
    throw new ReportError("inspection report missing 'header'");
  }
  const capabilities = requireArray(doc, "capabilities").map((g) => {
    if (!isObject(g)) throw new ReportError("capability group must be an object");
    const reqs = requireArray(g, "requirements").map((r) => {
      if (!isObject(r)) throw new ReportError("requirement must be an object");
      return { source: requireString(r, "source"), detail: requireString(r, "detail") };
    });
    return { domain: asDomain(g.domain), requirements: reqs };
  });

  // The parsed structure is validated field-by-field above; the cast is safe.
  return {
    schema: requireString(doc, "schema"),
    source: requireString(doc, "source"),
    header: {
      magic: requireString(header, "magic"),
      version: requireNumber(header, "version"),
      byte_length: requireNumber(header, "byte_length"),
      module_name:
        header.module_name === null ? null : String(header.module_name),
    },
