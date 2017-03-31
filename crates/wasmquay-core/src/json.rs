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
    /// All numbers are stored as `f64`; integers up to 2^53 round-trip exactly.
    Num(f64),
    /// A UTF-8 string.
    Str(String),
    /// An ordered array.
    Arr(Vec<Json>),
    /// An object with insertion-ordered key/value pairs.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Convenience constructor for a string value.
    pub fn s(v: impl Into<String>) -> Json {
        Json::Str(v.into())
    }

    /// Convenience constructor for an unsigned integer value.
    pub fn u(v: u64) -> Json {
