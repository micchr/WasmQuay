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

fn decode_imports(payload: &[u8]) -> Result<Vec<Import>> {
    let mut r = Reader::new(payload);
    let count = r.uleb128_u32()?;
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let module_name = r.name()?;
        let field = r.name()?;
        let kind = ExternalKind::from_byte(r.u8()?);
        // Consume the type descriptor so the cursor stays aligned.
        skip_import_desc(&mut r, kind)?;
        out.push(Import {
            module: module_name,
            field,
            kind,
        });
    }
    Ok(out)
}

fn skip_import_desc(r: &mut Reader<'_>, kind: ExternalKind) -> Result<()> {
    match kind {
        ExternalKind::Func => {
            // typeidx
            let _ = r.uleb128_u32()?;
        }
        ExternalKind::Table => {
            let _elem_type = r.u8()?;
            skip_limits(r)?;
        }
        ExternalKind::Memory => {
            skip_limits(r)?;
        }
        ExternalKind::Global => {
            let _valtype = r.u8()?;
            let _mutability = r.u8()?;
        }
        ExternalKind::Other(b) => {
            return Err(Error::new(
                ErrorKind::MalformedSection,
                format!("unsupported import kind byte 0x{:02x}", b),
            ));
        }
    }
    Ok(())
}

fn skip_limits(r: &mut Reader<'_>) -> Result<()> {
    let flags = r.u8()?;
    let _min = r.uleb128_u32()?;
    if flags & 0x01 != 0 {
        let _max = r.uleb128_u32()?;
    }
    Ok(())
}

fn decode_exports(payload: &[u8]) -> Result<Vec<Export>> {
    let mut r = Reader::new(payload);
    let count = r.uleb128_u32()?;
    let mut out = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let field = r.name()?;
        let kind = ExternalKind::from_byte(r.u8()?);
        let index = r.uleb128_u32()?;
        out.push(Export { field, kind, index });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_header() -> Vec<u8> {
        let mut v = WASM_MAGIC.to_vec();
        v.extend_from_slice(&1u32.to_le_bytes());
        v
    }

    #[test]
    fn rejects_bad_magic() {
        let err = parse(&[0, 0, 0, 0, 1, 0, 0, 0]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::BadMagic);
    }

    #[test]
    fn parses_empty_module() {
        let m = parse(&minimal_header()).unwrap();
        assert_eq!(m.version, 1);
        assert!(m.sections.is_empty());
    }

    #[test]
    fn parses_import_section() {
        let mut m = minimal_header();
        // import section id=2
        // body: count=1, module="wasi", field="fd_write", kind=func, typeidx=0
        let mut body = Vec::new();
        body.push(0x01); // count
        body.push(0x04);
        body.extend_from_slice(b"wasi");
        body.push(0x08);
        body.extend_from_slice(b"fd_write");
        body.push(0x00); // func kind
        body.push(0x00); // typeidx
        m.push(2);
        m.push(body.len() as u8);
        m.extend_from_slice(&body);

        let parsed = parse(&m).unwrap();
        assert_eq!(parsed.imports.len(), 1);
        assert_eq!(parsed.imports[0].module, "wasi");
        assert_eq!(parsed.imports[0].field, "fd_write");
        assert_eq!(parsed.imports[0].kind, ExternalKind::Func);
    }

    #[test]
    fn parses_export_and_name_sections() {
        let mut m = minimal_header();
        // export section id=7: count=1, field="run", kind=func, index=3
        let mut ex = Vec::new();
        ex.push(0x01);
        ex.push(0x03);
        ex.extend_from_slice(b"run");
        ex.push(0x00);
        ex.push(0x03);
        m.push(7);
        m.push(ex.len() as u8);
        m.extend_from_slice(&ex);

        // custom name section id=0: name="name", module subsection
        let mut namesec = Vec::new();
        namesec.push(0x04);
        namesec.extend_from_slice(b"name");
        // subsection 0 (module name)
        let mut sub0 = Vec::new();
        sub0.push(0x03);
        sub0.extend_from_slice(b"app");
        namesec.push(0x00);
        namesec.push(sub0.len() as u8);
        namesec.extend_from_slice(&sub0);
        m.push(0);
        m.push(namesec.len() as u8);
        m.extend_from_slice(&namesec);

        let parsed = parse(&m).unwrap();
        assert_eq!(parsed.exports.len(), 1);
        assert_eq!(parsed.exports[0].field, "run");
        assert_eq!(parsed.exports[0].index, 3);
        assert_eq!(parsed.module_name.as_deref(), Some("app"));
    }
}
