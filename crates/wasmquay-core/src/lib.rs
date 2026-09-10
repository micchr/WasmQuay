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
//! ## Scope & honesty
//!
//! wasmquay performs **static inspection and policy analysis**. It decodes the
//! binary and reasons about the capability surface a component declares through
//! its imports. It is **not** a WebAssembly runtime: it does not execute
//! modules, instantiate them, or enforce capabilities at run time. Where the
//! documentation talks about "enforcement" it means *analysis against a stated
//! policy*, not sandbox interception.
//!
//! ## Example
//!
//! ```
//! use wasmquay_core::{fixture::FixtureBuilder, wasm, policy, report};
//!
//! // Synthesize a module that imports a clock and a socket.
//! let bytes = FixtureBuilder::new()
//!     .module_name("svc")
//!     .import_func("wasi_snapshot_preview1", "clock_time_get")
//!     .import_func("wasi_snapshot_preview1", "sock_recv")
//!     .export_func("run", 0)
//!     .build();
//!
//! let module = wasm::parse(&bytes).unwrap();
//! let reqs = policy::requirements_from_module(&module);
//!
//! // Evaluate against a deny-by-default policy that only allows the clock.
//! let pol = policy::Policy::parse("policy \"p\"\ndefault deny\nallow clock\n").unwrap();
//! let eval = policy::evaluate(&reqs, &pol);
//!
//! assert!(!eval.is_compliant());               // network is denied
//! assert!(eval.violations().contains(&policy::Domain::Network));
//!
//! let json = report::evaluation_json(&eval).to_pretty();
//! assert!(json.contains("\"network\""));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod compat;
pub mod error;
pub mod fixture;
pub mod json;
pub mod leb;
pub mod policy;
pub mod report;
pub mod wasm;
pub mod wit;

pub use error::{Error, ErrorKind, Result};

/// The crate version, from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod integration {
    use super::*;

    /// End-to-end: build -> parse -> classify -> policy -> compat -> json.
    #[test]
    fn full_pipeline() {
        let a = fixture::FixtureBuilder::new()
            .module_name("a")
            .import_func("wasi_snapshot_preview1", "clock_time_get")
            .export_func("run", 0)
            .build();
        let b = fixture::FixtureBuilder::new()
            .module_name("b")
            .import_func("wasi_snapshot_preview1", "clock_time_get")
            .import_func("wasi_snapshot_preview1", "sock_recv")
            .export_func("run", 0)
            .build();

        let ma = wasm::parse(&a).unwrap();
        let mb = wasm::parse(&b).unwrap();

        let reqs_a = policy::requirements_from_module(&ma);
        assert_eq!(reqs_a.len(), 1);

        let pol = policy::Policy::parse("policy \"p\"\ndefault deny\nallow clock\n").unwrap();
        assert!(policy::evaluate(&reqs_a, &pol).is_compliant());

        // b adds a network capability -> not a safe replacement for a.
        let comp = compat::compare_modules(&ma, "a", &mb, "b");
        assert!(!comp.is_compatible());

        // Reports serialize without panicking and carry a schema.
        let insp = report::inspection_json(&ma, "a", &reqs_a).to_compact();
        assert!(insp.contains(report::SCHEMA_VERSION));
    }
}
