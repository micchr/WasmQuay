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
    sections: requireArray(doc, "sections").map((s) => {
      const so = asObject(s, "section");
      return {
        id: requireNumber(so, "id"),
        name: requireString(so, "name"),
        custom_name: so.custom_name === null ? null : String(so.custom_name),
        offset: requireNumber(so, "offset"),
        size: requireNumber(so, "size"),
      };
    }),
    imports: requireArray(doc, "imports").map((i) => {
      const io = asObject(i, "import");
      return {
        module: requireString(io, "module"),
        field: requireString(io, "field"),
        kind: requireString(io, "kind"),
      };
    }),
    exports: requireArray(doc, "exports").map((e) => {
      const eo = asObject(e, "export");
      return {
        field: requireString(eo, "field"),
        kind: requireString(eo, "kind"),
        index: requireNumber(eo, "index"),
      };
    }),
    names: requireArray(doc, "names").map((n) => {
      const no = asObject(n, "name");
      return { index: requireNumber(no, "index"), name: requireString(no, "name") };
    }),
    capabilities,
  };
}

/** Parse and validate a policy evaluation report. */
export function parsePolicy(text: string): PolicyReport {
  const doc = parseJson(text);
  if (!String(doc.schema).startsWith("wasmquay/policy")) {
    throw new ReportError(`not a policy report (schema='${String(doc.schema)}')`);
  }
  return {
    schema: requireString(doc, "schema"),
    policy: requireString(doc, "policy"),
    compliant: requireBool(doc, "compliant"),
    violations: requireArray(doc, "violations").map(asDomain),
    verdicts: requireArray(doc, "verdicts").map((v) => {
      const vo = asObject(v, "verdict");
      return {
        domain: asDomain(vo.domain),
        required: requireBool(vo, "required"),
        allowed: requireBool(vo, "allowed"),
        violation: requireBool(vo, "violation"),
        requirements: requireArray(vo, "requirements").map((r) => String(r)),
      };
    }),
  };
}

/** Parse and validate a compatibility report. */
export function parseCompat(text: string): CompatReport {
  const doc = parseJson(text);
  if (!String(doc.schema).startsWith("wasmquay/compat")) {
    throw new ReportError(`not a compat report (schema='${String(doc.schema)}')`);
  }
  return {
    schema: requireString(doc, "schema"),
    baseline: requireString(doc, "baseline"),
    candidate: requireString(doc, "candidate"),
    compatible: requireBool(doc, "compatible"),
    breaking_reasons: requireArray(doc, "breaking_reasons").map((r) => String(r)),
    export_diffs: requireArray(doc, "export_diffs").map((d) => {
      const do_ = asObject(d, "export_diff");
      return {
        change: asChange(do_.change),
        name: requireString(do_, "name"),
        kind: requireString(do_, "kind"),
      };
    }),
    capability_diffs: requireArray(doc, "capability_diffs").map((d) => {
      const do_ = asObject(d, "capability_diff");
      return {
        change: asChange(do_.change),
        domain: asDomain(do_.domain),
        source: requireString(do_, "source"),
      };
    }),
  };
}

function asChange(v: unknown): "added" | "removed" {
  if (v === "added" || v === "removed") return v;
  throw new ReportError(`change must be 'added' or 'removed', got '${String(v)}'`);
}

function asObject(v: unknown, what: string): Record<string, unknown> {
  if (!isObject(v)) throw new ReportError(`${what} must be an object`);
  return v;
}

function parseJson(text: string): Record<string, unknown> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (e) {
    throw new ReportError(`invalid JSON: ${(e as Error).message}`);
  }
  if (!isObject(parsed)) {
    throw new ReportError("report must be a JSON object");
  }
  return parsed;
}
