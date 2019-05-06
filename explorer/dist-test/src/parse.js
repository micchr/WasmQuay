/**
 * Runtime validation for wasmquay reports.
 *
 * `JSON.parse` yields `unknown`; these guards narrow it to the typed shapes in
 * `types.ts` and throw a descriptive `ReportError` when a document does not
 * match its declared schema. This keeps the rest of the explorer honest about
 * what it is consuming.
 */
import { ALL_DOMAINS } from "./types.js";
/** Thrown when a report does not conform to its expected schema. */
export class ReportError extends Error {
    constructor(message) {
        super(message);
        this.name = "ReportError";
    }
}
function isObject(v) {
    return typeof v === "object" && v !== null && !Array.isArray(v);
}
function requireString(obj, key) {
    const v = obj[key];
    if (typeof v !== "string") {
        throw new ReportError(`expected string field '${key}'`);
    }
    return v;
}
function requireNumber(obj, key) {
    const v = obj[key];
    if (typeof v !== "number") {
        throw new ReportError(`expected number field '${key}'`);
    }
    return v;
}
function requireBool(obj, key) {
    const v = obj[key];
    if (typeof v !== "boolean") {
        throw new ReportError(`expected boolean field '${key}'`);
    }
    return v;
}
function requireArray(obj, key) {
    const v = obj[key];
    if (!Array.isArray(v)) {
        throw new ReportError(`expected array field '${key}'`);
    }
    return v;
}
function asDomain(v) {
    if (typeof v === "string" && ALL_DOMAINS.includes(v)) {
        return v;
    }
    throw new ReportError(`unknown capability domain '${String(v)}'`);
}
/** Parse and validate an inspection report from a JSON string. */
export function parseInspection(text) {
    const doc = parseJson(text);
    if (!doc.schema || !String(doc.schema).startsWith("wasmquay/inspection")) {
        throw new ReportError(`not an inspection report (schema='${String(doc.schema)}')`);
    }
    const header = doc.header;
    if (!isObject(header)) {
        throw new ReportError("inspection report missing 'header'");
