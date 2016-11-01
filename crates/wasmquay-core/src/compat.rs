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
