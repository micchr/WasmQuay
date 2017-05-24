//! A cursor over a byte slice with the primitive decoders the WebAssembly
//! binary format needs: fixed-width little-endian integers, unsigned/signed
//! LEB128 variable-length integers, and length-prefixed UTF-8 names.
//!
//! All methods are bounds-checked and return a [`Result`]; they never panic on
//! malformed input.

use crate::error::{Error, ErrorKind, Result};

/// A forward-only reader over a byte slice.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// Create a reader over `data`, positioned at offset 0.
    pub fn new(data: &'a [u8]) -> Self {
