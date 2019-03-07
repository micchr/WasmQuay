/**
 * Static capability analysis over parsed wasmquay reports.
 *
 * The explorer's job is to turn a raw inspection (and optionally a policy
 * report) into an opinionated, human-facing risk view. Nothing here executes
 * WebAssembly — it is a pure function of the JSON the Rust CLI produced.
 */
import { ALL_DOMAINS } from "./types.js";
/** A coarse risk weight per capability domain (0 = inert, 5 = highest). */
export const DOMAIN_WEIGHT = {
    network: 5,
    fs: 4,
    unknown: 5,
    env: 2,
    random: 1,
    clock: 1,
    stdio: 1,
};
/** A short rationale shown alongside each domain in explanations. */
export const DOMAIN_RATIONALE = {
    network: "can open sockets / exfiltrate data",
    fs: "can read or modify files",
    unknown: "imports an unrecognized host — treat as untrusted",
    env: "can read environment variables and arguments",
    random: "consumes a randomness source",
    clock: "can read wall/monotonic clocks",
    stdio: "uses standard input/output streams",
};
/** Sum the weighted risk of the required domains in an inspection. */
export function riskScore(report) {
    let score = 0;
    for (const group of report.capabilities) {
        // Weight scales with the number of distinct entry points in a domain,
