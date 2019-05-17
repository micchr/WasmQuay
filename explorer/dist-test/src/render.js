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
