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
  );
  lines.push(
    `  imports=${surface.importCount} exports=${surface.exportCount}`,
  );
  if (surface.domains.length === 0) {
    lines.push("  capabilities: none — self-contained");
  } else {
    lines.push("  capabilities:");
    for (const domain of surface.domains) {
      const count = surface.counts[domain] ?? 0;
      lines.push(
        `    • ${domain.padEnd(8)} x${count}  — ${DOMAIN_RATIONALE[domain]}`,
      );
    }
  }
  return lines.join("\n");
}

/** Render reconciliation findings between an inspection and policy. */
export function renderFindings(findings: readonly Finding[]): string {
  if (findings.length === 0) {
    return "  (no capability findings)";
  }
  const glyph: Record<string, string> = {
    ok: "✓",
    note: "…",
    violation: "✗",
  };
  return findings
    .map((f) => `  ${glyph[f.severity]} [${f.severity}] ${f.message}`)
    .join("\n");
}

/** A machine-readable analysis summary for one component. */
export interface AnalysisSummary {
  readonly source: string;
  readonly moduleName: string | null;
  readonly score: number;
  readonly band: string;
  readonly domains: readonly string[];
