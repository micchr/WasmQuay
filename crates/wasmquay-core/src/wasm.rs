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
        1 => "type",
        2 => "import",
        3 => "function",
        4 => "table",
        5 => "memory",
        6 => "global",
        7 => "export",
        8 => "start",
        9 => "element",
        10 => "code",
        11 => "data",
        12 => "data-count",
        13 => "tag",
        _ => "unknown",
    }
}

/// A decoded section header: its id, byte range and (for custom sections) name.
#[derive(Debug, Clone)]
pub struct SectionInfo {
    /// The raw section id byte.
    pub id: u8,
    /// The standard section name for `id`.
    pub name: String,
    /// For a custom section, the declared custom name; otherwise empty.
    pub custom_name: String,
    /// Offset of the section *payload* (after id + size prefix).
    pub offset: usize,
    /// Size of the payload in bytes.
    pub size: usize,
}

/// A single import entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// The import module namespace (e.g. `wasi_snapshot_preview1`).
    pub module: String,
    /// The imported field/function name.
    pub field: String,
    /// The external kind of the import.
    pub kind: ExternalKind,
}

/// A single export entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    /// The exported name.
    pub field: String,
    /// The external kind of the export.
    pub kind: ExternalKind,
    /// The index into the corresponding index space.
    pub index: u32,
}

/// The fully decoded module.
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// The binary format version (little-endian `u32` from the header).
    pub version: u32,
    /// The decoded section table, in file order.
    pub sections: Vec<SectionInfo>,
    /// All import entries.
    pub imports: Vec<Import>,
    /// All export entries.
    pub exports: Vec<Export>,
    /// Module name from the custom `name` section, if present.
    pub module_name: Option<String>,
    /// Function index -> symbol name, from the custom `name` section.
    pub function_names: Vec<(u32, String)>,
    /// Total byte length of the input.
    pub byte_len: usize,
}

impl Module {
    /// Count of declared function-type imports.
    pub fn imported_functions(&self) -> usize {
        self.imports
            .iter()
            .filter(|i| i.kind == ExternalKind::Func)
            .count()
    }

    /// Count of imported memories (relevant to sandbox surface).
    pub fn imported_memories(&self) -> usize {
        self.imports
            .iter()
            .filter(|i| i.kind == ExternalKind::Memory)
            .count()
    }

    /// The distinct set of import module namespaces, sorted.
    pub fn import_namespaces(&self) -> Vec<String> {
        let mut ns: Vec<String> = self.imports.iter().map(|i| i.module.clone()).collect();
        ns.sort();
        ns.dedup();
        ns
    }
}

/// Parse a WebAssembly binary module from raw bytes.
pub fn parse(data: &[u8]) -> Result<Module> {
    let mut r = Reader::new(data);

    let magic = r.take(4)?;
    if magic != WASM_MAGIC {
        return Err(Error::at(
            ErrorKind::BadMagic,
            format!("expected \\0asm magic, found {:02x?}", magic),
            0,
        ));
    }
    let version = r.u32_le()?;
    if version == 0 {
        return Err(Error::at(
            ErrorKind::BadMagic,
            "version must be non-zero",
            4,
        ));
    }

    let mut module = Module {
        version,
        byte_len: data.len(),
        ..Module::default()
    };

    while !r.is_empty() {
        let id = r.u8()?;
        let size = r.uleb128_u32()? as usize;
        let payload_offset = r.position();
        let payload = r.take(size)?;

        let mut info = SectionInfo {
            id,
            name: section_name(id).to_string(),
            custom_name: String::new(),
            offset: payload_offset,
            size,
        };

        match id {
            0 => decode_custom(payload, &mut info, &mut module)?,
            2 => module.imports = decode_imports(payload)?,
            7 => module.exports = decode_exports(payload)?,
            // Other sections are recorded structurally but not deep-decoded.
            _ => {}
        }

        module.sections.push(info);
    }

    Ok(module)
}

fn decode_custom(payload: &[u8], info: &mut SectionInfo, module: &mut Module) -> Result<()> {
    let mut r = Reader::new(payload);
    let name = r.name()?;
    info.custom_name = name.clone();
    if name == "name" {
        decode_name_section(&mut r, module)?;
    }
    Ok(())
}

/// Decode the `name` custom section (subsections 0 = module, 1 = functions).
fn decode_name_section(r: &mut Reader<'_>, module: &mut Module) -> Result<()> {
    while !r.is_empty() {
        let subsection_id = r.u8()?;
        let sub_size = r.uleb128_u32()? as usize;
        let sub_bytes = r.take(sub_size)?;
        let mut sub = Reader::new(sub_bytes);
        match subsection_id {
            0 => {
                module.module_name = Some(sub.name()?);
            }
            1 => {
                let count = sub.uleb128_u32()?;
                for _ in 0..count {
                    let idx = sub.uleb128_u32()?;
                    let fname = sub.name()?;
                    module.function_names.push((idx, fname));
                }
            }
            // Local/label/type name subsections are skipped intentionally.
            _ => {}
        }
    }
    Ok(())
}

