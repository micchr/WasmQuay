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
