/**
 * Rendering helpers: turn analysis results into terminal-friendly text and a
 * compact JSON summary. Kept free of Node APIs so it can run in any host.
 */

import type { InspectionReport, PolicyReport } from "./types.js";
import {
  analyzeSurface,
  DOMAIN_RATIONALE,
  reconcile,
  type CapabilitySurface,
  type Finding,
} from "./analyze.js";

const BAND_GLYPH: Record<string, string> = {
  inert: "○",
  low: "◔",
  moderate: "◑",
  elevated: "◕",
