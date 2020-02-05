import { test } from "node:test";
import assert from "node:assert/strict";

import type { InspectionReport, PolicyReport } from "../src/types.js";
import {
  analyzeSurface,
  reconcile,
  riskBand,
  riskScore,
  rankSurfaces,
} from "../src/analyze.js";

function inspection(
  source: string,
  caps: { domain: string; count: number }[],
): InspectionReport {
  return {
    schema: "wasmquay/inspection@1",
    source,
