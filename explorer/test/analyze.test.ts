import { test } from "node:test";
import assert from "node:assert/strict";

import type { InspectionReport, PolicyReport } from "../src/types.js";
import {
  analyzeSurface,
  reconcile,
  riskBand,
  riskScore,
