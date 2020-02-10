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

