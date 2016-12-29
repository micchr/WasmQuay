//! Error types for the wasmquay core.
//!
//! Every fallible operation in the crate returns [`Result`], the alias built
//! on top of [`Error`]. The error type is deliberately small, `Clone` and
//! carries a human readable message plus a machine friendly [`ErrorKind`].

use std::fmt;

/// The category of a failure. Useful for programmatic handling and for
/// choosing an exit code in the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// The byte stream ended before a structure was fully decoded.
    UnexpectedEof,
    /// A magic number, version or section id was not what the spec requires.
    BadMagic,
