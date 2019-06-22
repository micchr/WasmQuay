import { test } from "node:test";
import assert from "node:assert/strict";
import { analyzeSurface, reconcile, riskBand, riskScore, rankSurfaces, } from "../src/analyze.js";
function inspection(source, caps) {
    return {
        schema: "wasmquay/inspection@1",
        source,
