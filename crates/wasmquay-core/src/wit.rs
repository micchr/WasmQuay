//! A reader for wasmquay's WIT-like interface manifest.
//!
//! The WebAssembly component model describes interfaces in WIT (Wasm Interface
//! Types). Rather than embed a full WIT parser, wasmquay defines a small,
//! well-specified textual subset that captures the pieces relevant to
//! capability analysis: the interfaces a component *imports* (its host
//! requirements) and the ones it *exports* (its public surface). The grammar
//! is documented in `docs/FORMAT.md`.
//!
//! Example manifest:
//!
//! ```text
//! package acme:image-tool@1.2.0
//!
//! world processor {
//!   import wasi:filesystem/types
//!   import wasi:clocks/wall-clock
//!   export process: func(input: list<u8>) -> list<u8>
//! }
//! ```
//!
//! The reader is line oriented, tolerant of blank lines and `//` comments, and
//! never panics on malformed input.

use crate::error::{Error, ErrorKind, Result};

/// One `import` or `export` declaration in a world.
#[derive(Debug, Clone, PartialEq, Eq)]
