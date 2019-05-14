/**
 * Rendering helpers: turn analysis results into terminal-friendly text and a
 * compact JSON summary. Kept free of Node APIs so it can run in any host.
 */
import { analyzeSurface, DOMAIN_RATIONALE, reconcile, } from "./analyze.js";
const BAND_GLYPH = {
