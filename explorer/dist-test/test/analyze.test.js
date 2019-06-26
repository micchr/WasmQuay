import { test } from "node:test";
import assert from "node:assert/strict";
import { analyzeSurface, reconcile, riskBand, riskScore, rankSurfaces, } from "../src/analyze.js";
function inspection(source, caps) {
    return {
        schema: "wasmquay/inspection@1",
        source,
        header: { magic: "\\0asm", version: 1, byte_length: 100, module_name: source },
        sections: [],
        imports: [],
        exports: [],
        names: [],
        capabilities: caps.map((c) => ({
            domain: c.domain,
            requirements: Array.from({ length: c.count }, (_, i) => ({
                source: "src",
                detail: `d${i}`,
            })),
        })),
    };
}
test("riskBand thresholds", () => {
    assert.equal(riskBand(0), "inert");
