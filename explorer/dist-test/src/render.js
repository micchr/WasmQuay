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
