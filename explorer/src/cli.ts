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
  wasmquay-explore surface <inspection.json> [policy.json] [--json]
  wasmquay-explore rank <inspection.json> [more.json ...] [--json]

FLAGS:
  --json   emit a machine-readable summary
  --help   show this help
`;

function fail(message: string, code: number): never {
  proc.stderr.write(`error: ${message}\n`);
  proc.exit(code);
}

function readReport(path: string): string {
  try {
    return readFileSync(path, "utf8");
  } catch (e) {
    fail(`cannot read '${path}': ${(e as Error).message}`, 2);
  }
}

function main(): void {
  const args = proc.argv.slice(2);
  if (args.length === 0 || args.includes("--help")) {
    proc.stdout.write(USAGE);
    proc.exit(0);
  }

  const asJson = args.includes("--json");
  const positional = args.filter((a) => !a.startsWith("--"));
  const command = positional[0];
  const rest = positional.slice(1);

  try {
