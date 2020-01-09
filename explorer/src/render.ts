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
  high: "●",
};

/** Render a capability surface as an aligned text block. */
export function renderSurface(surface: CapabilitySurface): string {
  const lines: string[] = [];
  const name = surface.moduleName ? ` (${surface.moduleName})` : "";
  lines.push(`▚ ${surface.source}${name}`);
  lines.push(
    `  risk: ${BAND_GLYPH[surface.band]} ${surface.band.toUpperCase()} (score ${surface.score})`,
