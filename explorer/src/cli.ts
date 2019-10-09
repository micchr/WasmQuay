#!/usr/bin/env node
/**
 * `wasmquay-explore` — the command-line wrapper around the explorer library.
 *
 * It reads wasmquay JSON reports from disk and prints a capability surface,
 * optionally reconciled against a policy report.
 *
 * USAGE:
 *   wasmquay-explore surface <inspection.json> [policy.json] [--json]
 *   wasmquay-explore rank <inspection.json> [<inspection.json> ...] [--json]
 *
 * Exit codes: 0 ok, 1 usage error, 2 read/parse error, 3 policy violation.
 */

import { readFileSync } from "node:fs";
import proc from "node:process";

import { parseInspection, parsePolicy, ReportError } from "./parse.js";
import { analyzeSurface, rankSurfaces } from "./analyze.js";
import { buildSummary, renderFindings, renderSurface } from "./render.js";

const USAGE = `wasmquay-explore — static capability explorer for wasmquay reports

USAGE:
