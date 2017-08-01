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

    /// Stable slug for reports and policy files.
    pub fn slug(self) -> &'static str {
        match self {
            Domain::Filesystem => "fs",
            Domain::Environment => "env",
            Domain::Clock => "clock",
            Domain::Network => "network",
            Domain::Random => "random",
            Domain::Stdio => "stdio",
            Domain::Unknown => "unknown",
        }
    }

    /// Parse a domain from its slug.
    pub fn from_slug(s: &str) -> Option<Domain> {
        Some(match s {
            "fs" => Domain::Filesystem,
            "env" => Domain::Environment,
            "clock" => Domain::Clock,
            "network" | "net" => Domain::Network,
            "random" | "rand" => Domain::Random,
            "stdio" => Domain::Stdio,
            "unknown" => Domain::Unknown,
            _ => return None,
        })
    }
}

/// A single classified capability requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The classified capability domain.
    pub domain: Domain,
    /// The originating import module namespace / interface path.
    pub source: String,
    /// The specific symbol / function that triggered the classification.
    pub detail: String,
}

/// Classify a WASI preview1 function name into a domain, if recognized.
fn classify_preview1(func: &str) -> Option<Domain> {
    let d = if func.starts_with("fd_") || func.starts_with("path_") {
        // fd_read/fd_write on stdio fds cannot be distinguished statically, so
        // generic fd_* / path_* are treated as filesystem access.
        Domain::Filesystem
    } else if func.starts_with("environ_") || func.starts_with("args_") {
        Domain::Environment
    } else if func.starts_with("clock_") {
        Domain::Clock
    } else if func.starts_with("sock_") {
        Domain::Network
    } else if func == "random_get" {
        Domain::Random
    } else {
        return None;
    };
    Some(d)
}

/// Classify a preview2 / component interface path into a domain.
fn classify_interface(path: &str) -> Option<Domain> {
