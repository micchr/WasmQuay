import { test } from "node:test";
import assert from "node:assert/strict";

import { parseInspection, parsePolicy, parseCompat, ReportError } from "../src/parse.js";

const INSPECTION = JSON.stringify({
  schema: "wasmquay/inspection@1",
  source: "net-service.wasm",
