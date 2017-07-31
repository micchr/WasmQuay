//! Capability classification and policy evaluation.
//!
//! A WebAssembly component only reaches the outside world through the host
//! functions it *imports*. wasmquay classifies each import into one of a small
//! number of capability **domains** and then checks the observed capability set
//! against an explicit [`Policy`] (allow / deny per domain, with optional
//! per-domain resource allow-lists).
//!
//! The classifier recognizes WASI preview1 (`wasi_snapshot_preview1`) function
//! names and WASI preview2 / component-model interface paths
//! (`wasi:filesystem/...`, `wasi:sockets/...`, etc.), and falls back to a
//! conservative "unknown host import" bucket so nothing is silently ignored.

use crate::error::{Error, ErrorKind, Result};
use crate::wasm::{ExternalKind, Module};
use crate::wit::Manifest;
use std::collections::BTreeMap;

/// The capability domains wasmquay reasons about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    /// Filesystem access (open, read, write, directory ops).
    Filesystem,
    /// Environment variables and program arguments.
    Environment,
    /// Wall clock / monotonic clock access.
    Clock,
    /// Network sockets (TCP/UDP, name resolution).
    Network,
    /// Randomness source.
    Random,
    /// Standard input/output/error streams.
    Stdio,
    /// A host import that could not be classified.
    Unknown,
}

impl Domain {
    /// All classifiable capability domains (excludes `Unknown`).
    pub const ALL: [Domain; 6] = [
        Domain::Filesystem,
        Domain::Environment,
        Domain::Clock,
        Domain::Network,
        Domain::Random,
        Domain::Stdio,
    ];

