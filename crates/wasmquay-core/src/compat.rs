//! Component compatibility comparison.
//!
//! Given two components (parsed modules and/or manifests) wasmquay computes a
//! substitutability report: can component *B* stand in for component *A*?
//!
//! The rule of thumb, borrowed from structural typing, is:
//!
//! * **Exports** are contravariant for the consumer: B must export at least
//!   everything A exports (B may export more).
//! * **Imports** are covariant for the host: B must not require any host
//!   capability that A did not already require (B may require fewer).
//!
//! We compare on the (name, kind) identity of exports and on the classified
//! capability domain plus source for imports.

use crate::policy::{requirements_from_module, Domain, Requirement};
use crate::wasm::{Export, Module};

/// The direction/severity of a single difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Present in the baseline but missing from the candidate.
    Removed,
    /// Present in the candidate but not the baseline.
    Added,
}

impl Change {
    /// Stable slug used in reports.
    pub fn slug(self) -> &'static str {
        match self {
            Change::Removed => "removed",
            Change::Added => "added",
        }
    }
}

/// A single export difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportDiff {
    /// Whether the export was added or removed.
    pub change: Change,
    /// The export name.
    pub name: String,
    /// The export kind slug.
    pub kind: String,
}

/// A single capability (import domain) difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDiff {
    /// Whether the capability was added or removed.
    pub change: Change,
    /// The affected capability domain.
    pub domain: Domain,
    /// The originating import module / interface path.
    pub source: String,
}

/// The full compatibility report between a baseline and a candidate.
#[derive(Debug, Clone)]
pub struct Compatibility {
    /// The baseline component's label.
    pub baseline_name: String,
    /// The candidate component's label.
    pub candidate_name: String,
    /// Differences in the export surface.
    pub export_diffs: Vec<ExportDiff>,
    /// Differences in the capability surface.
    pub capability_diffs: Vec<CapabilityDiff>,
}

impl Compatibility {
    /// The candidate is a safe drop-in replacement when:
    ///   * no export was removed, and
    ///   * no *new* capability domain was added.
    pub fn is_compatible(&self) -> bool {
        let export_ok = !self
            .export_diffs
            .iter()
            .any(|d| d.change == Change::Removed);
        let cap_ok = !self
            .capability_diffs
            .iter()
            .any(|d| d.change == Change::Added);
        export_ok && cap_ok
    }

    /// A short list of the reasons the candidate is *not* compatible.
    pub fn breaking_reasons(&self) -> Vec<String> {
        let mut reasons = Vec::new();
        for d in &self.export_diffs {
            if d.change == Change::Removed {
                reasons.push(format!("export removed: {} ({})", d.name, d.kind));
            }
        }
        for d in &self.capability_diffs {
            if d.change == Change::Added {
                reasons.push(format!(
                    "new capability required: {} via {}",
                    d.domain.slug(),
                    d.source
                ));
            }
        }
        reasons
    }
}

fn export_key(e: &Export) -> (String, String) {
    (e.field.clone(), e.kind.slug())
}

/// Compare two parsed modules for substitutability.
pub fn compare_modules(
    baseline: &Module,
    baseline_name: &str,
    candidate: &Module,
    candidate_name: &str,
) -> Compatibility {
    let export_diffs = diff_exports(&baseline.exports, &candidate.exports);
    let base_reqs = requirements_from_module(baseline);
    let cand_reqs = requirements_from_module(candidate);
    let capability_diffs = diff_capabilities(&base_reqs, &cand_reqs);

    Compatibility {
        baseline_name: baseline_name.to_string(),
        candidate_name: candidate_name.to_string(),
        export_diffs,
        capability_diffs,
    }
}

fn diff_exports(baseline: &[Export], candidate: &[Export]) -> Vec<ExportDiff> {
    let base_keys: Vec<(String, String)> = baseline.iter().map(export_key).collect();
    let cand_keys: Vec<(String, String)> = candidate.iter().map(export_key).collect();

    let mut diffs = Vec::new();
    for (name, kind) in &base_keys {
        if !cand_keys.iter().any(|k| k == &(name.clone(), kind.clone())) {
            diffs.push(ExportDiff {
                change: Change::Removed,
                name: name.clone(),
                kind: kind.clone(),
            });
        }
    }
    for (name, kind) in &cand_keys {
        if !base_keys.iter().any(|k| k == &(name.clone(), kind.clone())) {
            diffs.push(ExportDiff {
                change: Change::Added,
                name: name.clone(),
                kind: kind.clone(),
            });
        }
    }
    diffs.sort_by_key(|d| (d.name.clone(), d.kind.clone()));
    diffs
}

/// Compare capability requirements at the (domain, source) granularity.
fn diff_capabilities(baseline: &[Requirement], candidate: &[Requirement]) -> Vec<CapabilityDiff> {
    let base_set: Vec<(Domain, String)> = dedup(
        baseline
            .iter()
            .map(|r| (r.domain, r.source.clone()))
            .collect(),
    );
    let cand_set: Vec<(Domain, String)> = dedup(
        candidate
            .iter()
            .map(|r| (r.domain, r.source.clone()))
            .collect(),
    );

    let mut diffs = Vec::new();
    for (domain, source) in &cand_set {
        if !base_set.iter().any(|(d, s)| d == domain && s == source) {
            diffs.push(CapabilityDiff {
                change: Change::Added,
                domain: *domain,
                source: source.clone(),
            });
        }
    }
    for (domain, source) in &base_set {
        if !cand_set.iter().any(|(d, s)| d == domain && s == source) {
            diffs.push(CapabilityDiff {
                change: Change::Removed,
                domain: *domain,
                source: source.clone(),
            });
        }
    }
    diffs.sort_by_key(|d| (d.domain, d.source.clone()));
    diffs
}

fn dedup(mut v: Vec<(Domain, String)>) -> Vec<(Domain, String)> {
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::{ExternalKind, Import};

    fn export(name: &str) -> Export {
        Export {
            field: name.into(),
            kind: ExternalKind::Func,
            index: 0,
        }
    }

    fn module_with(exports: Vec<Export>, imports: Vec<Import>) -> Module {
        Module {
            exports,
            imports,
            ..Module::default()
        }
    }

    #[test]
    fn identical_modules_are_compatible() {
        let a = module_with(vec![export("run")], vec![]);
        let b = module_with(vec![export("run")], vec![]);
        let c = compare_modules(&a, "a", &b, "b");
        assert!(c.is_compatible());
        assert!(c.export_diffs.is_empty());
    }

    #[test]
    fn removed_export_breaks_compatibility() {
        let a = module_with(vec![export("run"), export("stop")], vec![]);
        let b = module_with(vec![export("run")], vec![]);
        let c = compare_modules(&a, "a", &b, "b");
        assert!(!c.is_compatible());
        assert!(c.breaking_reasons().iter().any(|r| r.contains("stop")));
    }

    #[test]
    fn added_export_is_compatible() {
        let a = module_with(vec![export("run")], vec![]);
        let b = module_with(vec![export("run"), export("extra")], vec![]);
        let c = compare_modules(&a, "a", &b, "b");
        assert!(c.is_compatible());
    }

    #[test]
    fn new_capability_breaks_compatibility() {
        let a = module_with(vec![export("run")], vec![]);
        let b = module_with(
            vec![export("run")],
            vec![Import {
                module: "wasi_snapshot_preview1".into(),
                field: "sock_recv".into(),
                kind: ExternalKind::Func,
            }],
        );
        let c = compare_modules(&a, "a", &b, "b");
        assert!(!c.is_compatible());
        assert!(c.breaking_reasons().iter().any(|r| r.contains("network")));
    }

    #[test]
    fn dropped_capability_is_compatible() {
        let a = module_with(
            vec![export("run")],
            vec![Import {
                module: "wasi_snapshot_preview1".into(),
                field: "clock_time_get".into(),
                kind: ExternalKind::Func,
            }],
        );
        let b = module_with(vec![export("run")], vec![]);
        let c = compare_modules(&a, "a", &b, "b");
        assert!(c.is_compatible());
    }
}
