/**
 * Static capability analysis over parsed wasmquay reports.
 *
 * The explorer's job is to turn a raw inspection (and optionally a policy
 * report) into an opinionated, human-facing risk view. Nothing here executes
 * WebAssembly — it is a pure function of the JSON the Rust CLI produced.
 */

import type {
  Domain,
  InspectionReport,
  PolicyReport,
} from "./types.js";
import { ALL_DOMAINS } from "./types.js";

/** A coarse risk weight per capability domain (0 = inert, 5 = highest). */
export const DOMAIN_WEIGHT: Readonly<Record<Domain, number>> = {
  network: 5,
  fs: 4,
  unknown: 5,
  env: 2,
  random: 1,
  clock: 1,
  stdio: 1,
};

/** A short rationale shown alongside each domain in explanations. */
export const DOMAIN_RATIONALE: Readonly<Record<Domain, string>> = {
  network: "can open sockets / exfiltrate data",
  fs: "can read or modify files",
  unknown: "imports an unrecognized host — treat as untrusted",
  env: "can read environment variables and arguments",
  random: "consumes a randomness source",
  clock: "can read wall/monotonic clocks",
  stdio: "uses standard input/output streams",
};

/** The risk band a component falls into. */
export type RiskBand = "inert" | "low" | "moderate" | "elevated" | "high";

/** The result of analyzing a single inspection report. */
export interface CapabilitySurface {
  readonly source: string;
  readonly moduleName: string | null;
  /** Domains actually required, sorted by descending weight. */
  readonly domains: readonly Domain[];
  /** The raw additive risk score. */
  readonly score: number;
  /** The bucketed risk band derived from the score. */
  readonly band: RiskBand;
  /** Per-domain counts of distinct requirements. */
  readonly counts: Readonly<Record<string, number>>;
  /** True when any import could not be classified. */
  readonly hasUnknown: boolean;
  readonly importCount: number;
  readonly exportCount: number;
}

/** Sum the weighted risk of the required domains in an inspection. */
export function riskScore(report: InspectionReport): number {
  let score = 0;
  for (const group of report.capabilities) {
    // Weight scales with the number of distinct entry points in a domain,
    // with diminishing returns so one noisy domain cannot dominate.
    const base = DOMAIN_WEIGHT[group.domain] ?? 0;
    const breadth = 1 + Math.log2(1 + group.requirements.length);
    score += base * breadth;
  }
  return Math.round(score * 10) / 10;
}

/** Bucket a raw score into a named band. */
export function riskBand(score: number): RiskBand {
  if (score <= 0) return "inert";
  if (score < 2) return "low";
  if (score < 6) return "moderate";
  if (score < 12) return "elevated";
  return "high";
}

/** Build the full capability surface for an inspection report. */
export function analyzeSurface(report: InspectionReport): CapabilitySurface {
  const counts: Record<string, number> = {};
  for (const group of report.capabilities) {
    counts[group.domain] = group.requirements.length;
  }

  const domains = report.capabilities
    .map((g) => g.domain)
    .slice()
    .sort((a, b) => (DOMAIN_WEIGHT[b] ?? 0) - (DOMAIN_WEIGHT[a] ?? 0));

  const score = riskScore(report);
  return {
    source: report.source,
    moduleName: report.header.module_name,
    domains,
    score,
    band: riskBand(score),
    counts,
    hasUnknown: report.capabilities.some((g) => g.domain === "unknown"),
    importCount: report.imports.length,
    exportCount: report.exports.length,
  };
}

/** A single reconciliation finding between an inspection and a policy. */
export interface Finding {
  readonly domain: Domain;
  readonly severity: "ok" | "note" | "violation";
  readonly message: string;
}

/**
 * Cross-check an inspection against a policy report for the same component.
 *
 * The policy report already contains the verdict, but reconciling the two
 * documents lets the explorer produce richer, per-requirement guidance and
 * surface disagreements (e.g. a domain required by the inspection but absent
 * from the policy verdicts, which would indicate mismatched inputs).
 */
export function reconcile(
  inspection: InspectionReport,
  policy: PolicyReport,
): Finding[] {
  const requiredDomains = new Set(inspection.capabilities.map((g) => g.domain));
  const findings: Finding[] = [];

  for (const domain of ALL_DOMAINS) {
    const verdict = policy.verdicts.find((v) => v.domain === domain);
    const requiredByInspection = requiredDomains.has(domain);

    if (verdict === undefined) {
      if (requiredByInspection) {
        findings.push({
          domain,
          severity: "note",
          message: `inspection requires '${domain}' but the policy report has no verdict for it (mismatched inputs?)`,
        });
      }
      continue;
    }

    if (verdict.violation) {
      findings.push({
        domain,
        severity: "violation",
        message: `'${domain}' is required but denied by policy '${policy.policy}' — ${DOMAIN_RATIONALE[domain]}`,
      });
    } else if (verdict.required) {
      findings.push({
        domain,
        severity: "ok",
        message: `'${domain}' required and allowed`,
      });
    }
  }

  return findings;
}

/** Rank several surfaces from most to least risky. */
export function rankSurfaces(
  surfaces: readonly CapabilitySurface[],
): CapabilitySurface[] {
  return surfaces.slice().sort((a, b) => b.score - a.score);
}
