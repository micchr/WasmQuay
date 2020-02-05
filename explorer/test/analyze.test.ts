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
  assert.equal(riskBand(8), "elevated");
  assert.equal(riskBand(20), "high");
});

test("network scores higher than clock", () => {
  const net = riskScore(inspection("net", [{ domain: "network", count: 1 }]));
  const clk = riskScore(inspection("clk", [{ domain: "clock", count: 1 }]));
  assert.ok(net > clk, `expected ${net} > ${clk}`);
});

