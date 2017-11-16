//! Report assembly: turn parsed modules, policies and comparisons into the
//! stable JSON/text documents consumed by the TypeScript explorer.
//!
//! The JSON schema is versioned via the top-level `schema` field and fully
//! documented in `docs/FORMAT.md`. Keeping serialization here (rather than
//! sprinkled through the parser) means the wire format has one owner.

use crate::compat::{Change, Compatibility};
use crate::json::Json;
use crate::policy::{Domain, Evaluation, Requirement};
use crate::wasm::Module;

/// The schema version emitted in every report. Bump on breaking changes.
pub const SCHEMA_VERSION: &str = "wasmquay/inspection@1";

/// Build the JSON report for a single inspected module.
pub fn inspection_json(module: &Module, source_label: &str, requirements: &[Requirement]) -> Json {
    Json::obj(vec![
        ("schema", Json::s(SCHEMA_VERSION)),
        ("source", Json::s(source_label)),
        ("header", header_json(module)),
        ("sections", sections_json(module)),
        ("imports", imports_json(module)),
        ("exports", exports_json(module)),
        ("names", names_json(module)),
        ("capabilities", capabilities_json(requirements)),
    ])
}

fn header_json(module: &Module) -> Json {
    Json::obj(vec![
        ("magic", Json::s("\\0asm")),
        ("version", Json::u(module.version as u64)),
        ("byte_length", Json::u(module.byte_len as u64)),
        (
            "module_name",
            match &module.module_name {
                Some(n) => Json::s(n.clone()),
                None => Json::Null,
            },
        ),
    ])
}

fn sections_json(module: &Module) -> Json {
    Json::Arr(
        module
            .sections
            .iter()
            .map(|s| {
                Json::obj(vec![
                    ("id", Json::u(s.id as u64)),
                    ("name", Json::s(s.name.clone())),
                    (
                        "custom_name",
                        if s.custom_name.is_empty() {
                            Json::Null
                        } else {
                            Json::s(s.custom_name.clone())
                        },
                    ),
                    ("offset", Json::u(s.offset as u64)),
                    ("size", Json::u(s.size as u64)),
                ])
            })
            .collect(),
    )
}

fn imports_json(module: &Module) -> Json {
    Json::Arr(
        module
            .imports
            .iter()
            .map(|i| {
                Json::obj(vec![
                    ("module", Json::s(i.module.clone())),
                    ("field", Json::s(i.field.clone())),
                    ("kind", Json::s(i.kind.slug())),
                ])
            })
            .collect(),
    )
}

fn exports_json(module: &Module) -> Json {
    Json::Arr(
        module
            .exports
            .iter()
            .map(|e| {
                Json::obj(vec![
                    ("field", Json::s(e.field.clone())),
                    ("kind", Json::s(e.kind.slug())),
                    ("index", Json::u(e.index as u64)),
                ])
            })
            .collect(),
    )
}

fn names_json(module: &Module) -> Json {
    Json::Arr(
        module
            .function_names
            .iter()
            .map(|(idx, name)| {
                Json::obj(vec![
                    ("index", Json::u(*idx as u64)),
                    ("name", Json::s(name.clone())),
                ])
            })
            .collect(),
    )
}

