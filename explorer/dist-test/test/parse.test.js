import { test } from "node:test";
import assert from "node:assert/strict";
import { parseInspection, parsePolicy, parseCompat, ReportError } from "../src/parse.js";
const INSPECTION = JSON.stringify({
    schema: "wasmquay/inspection@1",
    source: "net-service.wasm",
    header: { magic: "\\0asm", version: 1, byte_length: 208, module_name: "net-service" },
