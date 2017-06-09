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
        Reader { data, pos: 0 }
    }

    /// Current absolute byte offset.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Number of bytes remaining.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// True when the cursor has consumed all input.
    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    /// Read a single byte.
    pub fn u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::at(
                ErrorKind::UnexpectedEof,
                "expected a byte",
                self.pos,
            ));
        }
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    /// Read a fixed little-endian `u32` (4 bytes).
    pub fn u32_le(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Borrow the next `n` bytes and advance.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| Error::at(ErrorKind::MalformedSection, "length overflow", self.pos))?;
        if end > self.data.len() {
            return Err(Error::at(
