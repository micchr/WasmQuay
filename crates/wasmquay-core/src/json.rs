//! A tiny, dependency-free JSON value model and serializer.
//!
//! The toolkit emits machine-readable reports but does not pull in `serde`
//! (to keep the offline, zero-dependency promise). This module provides an
//! ordered [`Json`] value type and a deterministic pretty/compact printer.
//!
//! Object keys preserve insertion order so diffs between report runs are
//! stable and readable.

use std::fmt::Write as _;

/// An in-memory JSON value. Objects retain insertion order.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// The JSON `null` literal.
    Null,
    /// A boolean.
    Bool(bool),
