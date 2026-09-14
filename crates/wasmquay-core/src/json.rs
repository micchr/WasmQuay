//! A tiny, dependency-free JSON value model and serializer.
//!
//! The toolkit emits machine-readable reports but does not pull in `serde`
//! (to keep the offline, zero-dependency promise). This module provides an
//! ordered [`Json`] value type and a deterministic pretty/compact printer.
//!
//! Object keys preserve insertion order so diffs between report runs are
//! stable and readable.

use std::fmt::Write as _;

/// An in-memory JSON value. Objects retain insertion order.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    /// The JSON `null` literal.
    Null,
    /// A boolean.
    Bool(bool),
    /// All numbers are stored as `f64`; integers up to 2^53 round-trip exactly.
    Num(f64),
    /// A UTF-8 string.
    Str(String),
    /// An ordered array.
    Arr(Vec<Json>),
    /// An object with insertion-ordered key/value pairs.
    Obj(Vec<(String, Json)>),
}

impl Json {
    /// Convenience constructor for a string value.
    pub fn s(v: impl Into<String>) -> Json {
        Json::Str(v.into())
    }

    /// Convenience constructor for an unsigned integer value.
    pub fn u(v: u64) -> Json {
        Json::Num(v as f64)
    }

    /// Build an object from key/value pairs, preserving order.
    pub fn obj(pairs: Vec<(&str, Json)>) -> Json {
        Json::Obj(pairs.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }

    /// Build an array from a vector of strings.
    pub fn str_arr<I: IntoIterator<Item = String>>(items: I) -> Json {
        Json::Arr(items.into_iter().map(Json::Str).collect())
    }

    /// Serialize compactly (no whitespace).
    pub fn to_compact(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, None, 0);
        out
    }

    /// Serialize with two-space indentation.
    pub fn to_pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, Some(2), 0);
        out
    }

    fn write(&self, out: &mut String, indent: Option<usize>, depth: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Json::Num(n) => write_number(out, *n),
            Json::Str(s) => write_json_string(out, s),
            Json::Arr(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline_indent(out, indent, depth + 1);
                    item.write(out, indent, depth + 1);
                }
                newline_indent(out, indent, depth);
                out.push(']');
            }
            Json::Obj(pairs) => {
                if pairs.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    newline_indent(out, indent, depth + 1);
                    write_json_string(out, k);
                    out.push(':');
                    if indent.is_some() {
                        out.push(' ');
                    }
                    v.write(out, indent, depth + 1);
                }
                newline_indent(out, indent, depth);
                out.push('}');
            }
        }
    }
}

fn newline_indent(out: &mut String, indent: Option<usize>, depth: usize) {
    if let Some(step) = indent {
        out.push('\n');
        for _ in 0..(step * depth) {
            out.push(' ');
        }
    }
}

fn write_number(out: &mut String, n: f64) {
    if n.is_finite() {
        if n.fract() == 0.0 && n.abs() < 9.007_199_254_740_992e15 {
            let _ = write!(out, "{}", n as i64);
        } else {
            let _ = write!(out, "{}", n);
        }
    } else {
        // JSON has no NaN/Infinity; emit null to keep output valid.
        out.push_str("null");
    }
}

fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_object_preserves_order() {
        let j = Json::obj(vec![("b", Json::u(2)), ("a", Json::s("x"))]);
        assert_eq!(j.to_compact(), r#"{"b":2,"a":"x"}"#);
    }

    #[test]
    fn escapes_control_and_quotes() {
        let j = Json::s("line\n\"q\"\t\u{1}");
        assert_eq!(j.to_compact(), r#""line\n\"q\"\t\u0001""#);
    }

    #[test]
    fn integers_render_without_decimal() {
        assert_eq!(Json::u(42).to_compact(), "42");
        assert_eq!(Json::Num(3.5).to_compact(), "3.5");
    }

    #[test]
    fn pretty_indents_nested() {
        let j = Json::obj(vec![("k", Json::Arr(vec![Json::u(1), Json::u(2)]))]);
        let pretty = j.to_pretty();
        assert!(pretty.contains("\n  \"k\""));
        assert!(pretty.contains("\n    1"));
    }

    #[test]
    fn empty_containers() {
        assert_eq!(Json::Arr(vec![]).to_pretty(), "[]");
        assert_eq!(Json::Obj(vec![]).to_pretty(), "{}");
    }
}
