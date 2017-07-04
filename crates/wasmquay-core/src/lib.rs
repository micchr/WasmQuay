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
//! 3. [`wit`] — a reader for wasmquay's WIT-like interface manifest subset.
//! 4. [`policy`] — capability classification and explicit policy evaluation.
//! 5. [`compat`] — substitutability comparison between two components.
//! 6. [`report`] — stable JSON/text serialization consumed downstream.
//! 7. [`fixture`] — a matching binary *encoder* for deterministic test data.
//!
