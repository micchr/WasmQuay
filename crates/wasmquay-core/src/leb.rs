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
                ErrorKind::UnexpectedEof,
                format!("wanted {} bytes, {} remain", n, self.remaining()),
                self.pos,
            ));
        }
        let slice = &self.data[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    /// Skip `n` bytes.
    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.take(n).map(|_| ())
    }

    /// Decode an unsigned LEB128 integer (max 64 bits).
    pub fn uleb128(&mut self) -> Result<u64> {
        let start = self.pos;
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.u8()?;
            if shift >= 64 {
                return Err(Error::at(ErrorKind::BadLeb128, "uleb128 too long", start));
            }
            let low = (byte & 0x7f) as u64;
            // Guard against bits that would overflow the 64-bit accumulator.
            if shift == 63 && low > 1 {
                return Err(Error::at(ErrorKind::BadLeb128, "uleb128 overflow", start));
            }
            result |= low << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        Ok(result)
    }

    /// Decode an unsigned LEB128 integer that must fit in a `u32`.
    pub fn uleb128_u32(&mut self) -> Result<u32> {
        let v = self.uleb128()?;
        u32::try_from(v).map_err(|_| Error::at(ErrorKind::BadLeb128, "value exceeds u32", self.pos))
    }

    /// Decode a signed LEB128 integer (max 64 bits).
    pub fn sleb128(&mut self) -> Result<i64> {
