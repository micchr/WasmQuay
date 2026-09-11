//! A small WebAssembly binary *encoder* used to synthesize deterministic test
//! fixtures without any external toolchain.
//!
//! It writes the same binary grammar the [`crate::wasm`] decoder reads, so the
//! round-trip (`build` -> `parse`) is an end-to-end test of the format code.
//! The encoder covers exactly the surface wasmquay inspects: a header, an
//! import section, an export section and a custom `name` section. It is not a
//! general-purpose assembler.

use crate::wasm::{ExternalKind, WASM_MAGIC};

/// Builder for a synthetic module.
#[derive(Debug, Default, Clone)]
pub struct FixtureBuilder {
    module_name: Option<String>,
    imports: Vec<(String, String, ExternalKind)>,
    exports: Vec<(String, ExternalKind, u32)>,
    function_names: Vec<(u32, String)>,
}

impl FixtureBuilder {
    /// Start a new fixture.
    pub fn new() -> Self {
        FixtureBuilder::default()
    }

    /// Set the module name emitted in the custom `name` section.
    pub fn module_name(mut self, name: &str) -> Self {
        self.module_name = Some(name.to_string());
        self
    }

    /// Add a function import `module.field`.
    pub fn import_func(mut self, module: &str, field: &str) -> Self {
        self.imports
            .push((module.to_string(), field.to_string(), ExternalKind::Func));
        self
    }

    /// Add a memory import (data plumbing, not a capability).
    pub fn import_memory(mut self, module: &str, field: &str) -> Self {
        self.imports
            .push((module.to_string(), field.to_string(), ExternalKind::Memory));
        self
    }

    /// Add a function export.
    pub fn export_func(mut self, field: &str, index: u32) -> Self {
        self.exports
            .push((field.to_string(), ExternalKind::Func, index));
        self
    }

    /// Add a function-name mapping for the custom `name` section.
    pub fn function_name(mut self, index: u32, name: &str) -> Self {
        self.function_names.push((index, name.to_string()));
        self
    }

    /// Encode the module to a byte vector.
    pub fn build(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&WASM_MAGIC);
        out.extend_from_slice(&1u32.to_le_bytes());

        if !self.imports.is_empty() {
            let body = self.encode_import_section();
            emit_section(&mut out, 2, &body);
        }
        if !self.exports.is_empty() {
            let body = self.encode_export_section();
            emit_section(&mut out, 7, &body);
        }
        if self.module_name.is_some() || !self.function_names.is_empty() {
            let body = self.encode_name_section();
            emit_section(&mut out, 0, &body);
        }

        out
    }

    fn encode_import_section(&self) -> Vec<u8> {
        let mut body = Vec::new();
        write_uleb(&mut body, self.imports.len() as u64);
        for (module, field, kind) in &self.imports {
            write_name(&mut body, module);
            write_name(&mut body, field);
            match kind {
                ExternalKind::Func => {
                    body.push(0x00);
                    write_uleb(&mut body, 0); // typeidx 0
                }
                ExternalKind::Memory => {
                    body.push(0x02);
                    body.push(0x00); // limits flags: min only
                    write_uleb(&mut body, 1); // min 1 page
                }
                ExternalKind::Table => {
                    body.push(0x01);
                    body.push(0x70); // funcref
                    body.push(0x00);
                    write_uleb(&mut body, 1);
                }
                ExternalKind::Global => {
                    body.push(0x03);
                    body.push(0x7f); // i32
                    body.push(0x00); // immutable
                }
                ExternalKind::Other(b) => body.push(*b),
            }
        }
        body
    }

    fn encode_export_section(&self) -> Vec<u8> {
        let mut body = Vec::new();
        write_uleb(&mut body, self.exports.len() as u64);
        for (field, kind, index) in &self.exports {
            write_name(&mut body, field);
            let kb = match kind {
                ExternalKind::Func => 0x00,
                ExternalKind::Table => 0x01,
                ExternalKind::Memory => 0x02,
                ExternalKind::Global => 0x03,
                ExternalKind::Other(b) => *b,
            };
            body.push(kb);
            write_uleb(&mut body, *index as u64);
        }
        body
    }

    fn encode_name_section(&self) -> Vec<u8> {
        let mut body = Vec::new();
        write_name(&mut body, "name");
        if let Some(name) = &self.module_name {
            let mut sub = Vec::new();
            write_name(&mut sub, name);
            body.push(0x00); // subsection: module name
            write_uleb(&mut body, sub.len() as u64);
            body.extend_from_slice(&sub);
        }
        if !self.function_names.is_empty() {
            let mut sub = Vec::new();
            write_uleb(&mut sub, self.function_names.len() as u64);
            for (idx, fname) in &self.function_names {
                write_uleb(&mut sub, *idx as u64);
                write_name(&mut sub, fname);
            }
            body.push(0x01); // subsection: function names
            write_uleb(&mut body, sub.len() as u64);
            body.extend_from_slice(&sub);
        }
        body
    }
}

fn emit_section(out: &mut Vec<u8>, id: u8, body: &[u8]) {
    out.push(id);
    write_uleb(out, body.len() as u64);
    out.extend_from_slice(body);
}

fn write_uleb(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn write_name(out: &mut Vec<u8>, name: &str) {
    write_uleb(out, name.len() as u64);
    out.extend_from_slice(name.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::parse;

    #[test]
    fn round_trips_through_parser() {
        let bytes = FixtureBuilder::new()
            .module_name("demo")
            .import_func("wasi_snapshot_preview1", "clock_time_get")
            .import_func("wasi_snapshot_preview1", "fd_write")
            .import_memory("env", "memory")
            .export_func("run", 3)
            .function_name(3, "run")
            .build();

        let m = parse(&bytes).unwrap();
        assert_eq!(m.module_name.as_deref(), Some("demo"));
        assert_eq!(m.imports.len(), 3);
        assert_eq!(m.imported_functions(), 2);
        assert_eq!(m.imported_memories(), 1);
        assert_eq!(m.exports.len(), 1);
        assert_eq!(m.exports[0].field, "run");
        assert_eq!(m.function_names, vec![(3, "run".to_string())]);
    }

    #[test]
    fn large_uleb_length_prefixes_round_trip() {
        // Build a module with an import whose field name forces a 2-byte uleb.
        let long = "x".repeat(200);
        let bytes = FixtureBuilder::new().import_func("host", &long).build();
        let m = parse(&bytes).unwrap();
        assert_eq!(m.imports[0].field.len(), 200);
    }
}
