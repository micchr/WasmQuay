//! WebAssembly binary decoder.
//!
//! This module parses the parts of a `.wasm` module (WebAssembly binary
//! format, "core" modules) that matter for capability inspection:
//!
//! * the 8-byte header (`\0asm` magic + version);
//! * the section table with ids, sizes and byte ranges;
//! * the **import** section (module/name/kind), which is where a component's
//!   host requirements — and therefore its capability surface — live;
//! * the **export** section (name/kind);
//! * the custom **name** section (`0` module name, `1` function names),
//!   used to render friendly symbol names.
//!
//! We follow the binary grammar from the WebAssembly core specification. The
//! decoder is total: malformed input yields an [`Error`], never a panic.
//!
//! Note on scope: this decodes *core* WebAssembly modules. The nascent
//! component-model binary layout reuses the same section framing, so section
//! walking and custom-name extraction work on those too, but we do not claim
//! to decode component-model type definitions here — the WIT-like manifest
//! reader ([`crate::wit`]) covers the interface layer instead.

use crate::error::{Error, ErrorKind, Result};
use crate::leb::Reader;

/// The 4-byte magic that opens every WebAssembly binary: `\0asm`.
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// External kind of an import or export entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalKind {
    /// A function.
    Func,
    /// A table (typically of function references).
    Table,
    /// A linear memory.
    Memory,
    /// A global variable.
    Global,
    /// Reserved/unknown kind byte (component-model or future extension).
    Other(u8),
}

impl ExternalKind {
    fn from_byte(b: u8) -> ExternalKind {
        match b {
            0x00 => ExternalKind::Func,
            0x01 => ExternalKind::Table,
            0x02 => ExternalKind::Memory,
            0x03 => ExternalKind::Global,
            other => ExternalKind::Other(other),
        }
    }

    /// Stable slug used in reports.
    pub fn slug(self) -> String {
        match self {
            ExternalKind::Func => "func".into(),
            ExternalKind::Table => "table".into(),
            ExternalKind::Memory => "memory".into(),
            ExternalKind::Global => "global".into(),
            ExternalKind::Other(b) => format!("other:0x{:02x}", b),
        }
    }
}

/// Human-readable name for a standard section id.
pub fn section_name(id: u8) -> &'static str {
    match id {
        0 => "custom",
