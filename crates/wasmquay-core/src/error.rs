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
    /// A LEB128 integer was malformed (too long / overflowing).
    BadLeb128,
    /// A UTF-8 name inside the module was not valid UTF-8.
    BadUtf8,
    /// A section or subsection declared a size that does not line up.
    MalformedSection,
    /// A manifest (WIT-like or policy) could not be parsed.
    BadManifest,
    /// An input/output error while reading a file.
    Io,
}

impl ErrorKind {
    /// A stable, lower-case slug for the kind. Emitted in JSON reports.
    pub fn slug(self) -> &'static str {
        match self {
            ErrorKind::UnexpectedEof => "unexpected-eof",
            ErrorKind::BadMagic => "bad-magic",
            ErrorKind::BadLeb128 => "bad-leb128",
            ErrorKind::BadUtf8 => "bad-utf8",
            ErrorKind::MalformedSection => "malformed-section",
            ErrorKind::BadManifest => "bad-manifest",
            ErrorKind::Io => "io",
        }
    }
}

/// A wasmquay error: a [`ErrorKind`] plus a descriptive message and an
/// optional byte offset into the source that triggered it.
#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
    offset: Option<usize>,
}

impl Error {
    /// Build an error with a message.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Error {
            kind,
            message: message.into(),
            offset: None,
        }
    }

    /// Build an error that points at a byte offset.
    pub fn at(kind: ErrorKind, message: impl Into<String>, offset: usize) -> Self {
        Error {
            kind,
            message: message.into(),
            offset: Some(offset),
        }
    }

    /// The category of this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The offset into the source, if known.
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }
