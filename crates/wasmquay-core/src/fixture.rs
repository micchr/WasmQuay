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
