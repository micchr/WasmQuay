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
