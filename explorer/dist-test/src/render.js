/**
 * Rendering helpers: turn analysis results into terminal-friendly text and a
 * compact JSON summary. Kept free of Node APIs so it can run in any host.
 */
import { analyzeSurface, DOMAIN_RATIONALE, reconcile, } from "./analyze.js";
const BAND_GLYPH = {
    inert: "○",
    low: "◔",
    moderate: "◑",
    elevated: "◕",
    high: "●",
};
/** Render a capability surface as an aligned text block. */
export function renderSurface(surface) {
    const lines = [];
    const name = surface.moduleName ? ` (${surface.moduleName})` : "";
    lines.push(`▚ ${surface.source}${name}`);
    lines.push(`  risk: ${BAND_GLYPH[surface.band]} ${surface.band.toUpperCase()} (score ${surface.score})`);
    lines.push(`  imports=${surface.importCount} exports=${surface.exportCount}`);
    if (surface.domains.length === 0) {
        lines.push("  capabilities: none — self-contained");
    }
    else {
        lines.push("  capabilities:");
        for (const domain of surface.domains) {
            const count = surface.counts[domain] ?? 0;
            lines.push(`    • ${domain.padEnd(8)} x${count}  — ${DOMAIN_RATIONALE[domain]}`);
        }
    }
    return lines.join("\n");
}
/** Render reconciliation findings between an inspection and policy. */
export function renderFindings(findings) {
    if (findings.length === 0) {
        return "  (no capability findings)";
    }
    const glyph = {
        ok: "✓",
        note: "…",
        violation: "✗",
    };
    return findings
        .map((f) => `  ${glyph[f.severity]} [${f.severity}] ${f.message}`)
        .join("\n");
}
/** Build the JSON summary object for an inspection (+ optional policy). */
export function buildSummary(inspection, policy) {
    const surface = analyzeSurface(inspection);
    const base = {
        source: surface.source,
        moduleName: surface.moduleName,
        score: surface.score,
        band: surface.band,
        domains: surface.domains,
        hasUnknown: surface.hasUnknown,
    };
    if (policy) {
        return {
            ...base,
            findings: reconcile(inspection, policy),
            compliant: policy.compliant,
        };
    }
    return base;
}
