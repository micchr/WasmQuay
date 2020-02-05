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
    header: { magic: "\\0asm", version: 1, byte_length: 100, module_name: source },
    sections: [],
    imports: [],
    exports: [],
    names: [],
    capabilities: caps.map((c) => ({
      domain: c.domain as InspectionReport["capabilities"][number]["domain"],
      requirements: Array.from({ length: c.count }, (_, i) => ({
        source: "src",
        detail: `d${i}`,
      })),
    })),
  };
}

test("riskBand thresholds", () => {
  assert.equal(riskBand(0), "inert");
  assert.equal(riskBand(1), "low");
  assert.equal(riskBand(4), "moderate");
