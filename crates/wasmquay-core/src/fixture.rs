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

