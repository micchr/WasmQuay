//! # wasmquay-core
//!
//! Offline, zero-dependency core for inspecting WebAssembly components and
//! evaluating capability policies.
//!
//! The crate is organized as a pipeline:
//!
//! 1. [`leb`] — primitive byte / LEB128 decoding.
//! 2. [`wasm`] — the WebAssembly binary decoder (header, sections, imports,
//!    exports, custom `name` section).
