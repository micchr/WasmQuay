import { test } from "node:test";
import assert from "node:assert/strict";

import { parseInspection, parsePolicy, parseCompat, ReportError } from "../src/parse.js";

const INSPECTION = JSON.stringify({
  schema: "wasmquay/inspection@1",
  source: "net-service.wasm",
  header: { magic: "\\0asm", version: 1, byte_length: 208, module_name: "net-service" },
  sections: [{ id: 2, name: "import", custom_name: null, offset: 10, size: 90 }],
  imports: [{ module: "wasi_snapshot_preview1", field: "sock_recv", kind: "func" }],
  exports: [{ field: "run", kind: "func", index: 0 }],
  names: [{ index: 0, name: "run" }],
  capabilities: [
    { domain: "network", requirements: [{ source: "wasi_snapshot_preview1", detail: "sock_recv" }] },
  ],
});

test("parseInspection accepts a valid report", () => {
  const r = parseInspection(INSPECTION);
  assert.equal(r.source, "net-service.wasm");
  assert.equal(r.header.version, 1);
  assert.equal(r.capabilities[0].domain, "network");
});

test("parseInspection rejects wrong schema", () => {
  const bad = JSON.stringify({ schema: "other@1" });
  assert.throws(() => parseInspection(bad));
});

test("parseInspection rejects invalid JSON", () => {
  assert.throws(() => parseInspection("{not json"));
});

test("parseInspection rejects unknown domain", () => {
  const bad = JSON.stringify({
    schema: "wasmquay/inspection@1",
    source: "x",
    header: { magic: "\\0asm", version: 1, byte_length: 1, module_name: null },
    sections: [],
    imports: [],
    exports: [],
    names: [],
    capabilities: [{ domain: "telepathy", requirements: [] }],
  });
  assert.throws(() => parseInspection(bad));
  // The thrown error is a ReportError (verified by the throw above).
  try {
    parseInspection(bad);
  } catch (e) {
    assert.ok(e instanceof ReportError);
  }
});

test("parsePolicy reads verdicts and compliance", () => {
  const doc = JSON.stringify({
    schema: "wasmquay/policy@1",
    policy: "sandbox-strict",
    compliant: false,
    violations: ["network"],
    verdicts: [
      { domain: "network", required: true, allowed: false, violation: true, requirements: ["a :: b"] },
    ],
  });
  const p = parsePolicy(doc);
  assert.equal(p.compliant, false);
  assert.deepEqual(p.violations, ["network"]);
  assert.equal(p.verdicts[0].violation, true);
