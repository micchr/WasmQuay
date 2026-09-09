//! Capability classification and policy evaluation.
//!
//! A WebAssembly component only reaches the outside world through the host
//! functions it *imports*. wasmquay classifies each import into one of a small
//! number of capability **domains** and then checks the observed capability set
//! against an explicit [`Policy`] (allow / deny per domain, with optional
//! per-domain resource allow-lists).
//!
//! The classifier recognizes WASI preview1 (`wasi_snapshot_preview1`) function
//! names and WASI preview2 / component-model interface paths
//! (`wasi:filesystem/...`, `wasi:sockets/...`, etc.), and falls back to a
//! conservative "unknown host import" bucket so nothing is silently ignored.

use crate::error::{Error, ErrorKind, Result};
use crate::wasm::{ExternalKind, Module};
use crate::wit::Manifest;
use std::collections::BTreeMap;

/// The capability domains wasmquay reasons about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    /// Filesystem access (open, read, write, directory ops).
    Filesystem,
    /// Environment variables and program arguments.
    Environment,
    /// Wall clock / monotonic clock access.
    Clock,
    /// Network sockets (TCP/UDP, name resolution).
    Network,
    /// Randomness source.
    Random,
    /// Standard input/output/error streams.
    Stdio,
    /// A host import that could not be classified.
    Unknown,
}

impl Domain {
    /// All classifiable capability domains (excludes `Unknown`).
    pub const ALL: [Domain; 6] = [
        Domain::Filesystem,
        Domain::Environment,
        Domain::Clock,
        Domain::Network,
        Domain::Random,
        Domain::Stdio,
    ];

    /// Stable slug for reports and policy files.
    pub fn slug(self) -> &'static str {
        match self {
            Domain::Filesystem => "fs",
            Domain::Environment => "env",
            Domain::Clock => "clock",
            Domain::Network => "network",
            Domain::Random => "random",
            Domain::Stdio => "stdio",
            Domain::Unknown => "unknown",
        }
    }

    /// Parse a domain from its slug.
    pub fn from_slug(s: &str) -> Option<Domain> {
        Some(match s {
            "fs" => Domain::Filesystem,
            "env" => Domain::Environment,
            "clock" => Domain::Clock,
            "network" | "net" => Domain::Network,
            "random" | "rand" => Domain::Random,
            "stdio" => Domain::Stdio,
            "unknown" => Domain::Unknown,
            _ => return None,
        })
    }
}

/// A single classified capability requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The classified capability domain.
    pub domain: Domain,
    /// The originating import module namespace / interface path.
    pub source: String,
    /// The specific symbol / function that triggered the classification.
    pub detail: String,
}

/// Classify a WASI preview1 function name into a domain, if recognized.
fn classify_preview1(func: &str) -> Option<Domain> {
    let d = if func.starts_with("fd_") || func.starts_with("path_") {
        // fd_read/fd_write on stdio fds cannot be distinguished statically, so
        // generic fd_* / path_* are treated as filesystem access.
        Domain::Filesystem
    } else if func.starts_with("environ_") || func.starts_with("args_") {
        Domain::Environment
    } else if func.starts_with("clock_") {
        Domain::Clock
    } else if func.starts_with("sock_") {
        Domain::Network
    } else if func == "random_get" {
        Domain::Random
    } else {
        return None;
    };
    Some(d)
}

/// Classify a preview2 / component interface path into a domain.
fn classify_interface(path: &str) -> Option<Domain> {
    // Paths look like `wasi:filesystem/types` or `wasi:sockets/tcp`.
    let body = path.strip_prefix("wasi:").unwrap_or(path);
    let head = body.split(['/', '@']).next().unwrap_or(body);
    Some(match head {
        "filesystem" => Domain::Filesystem,
        "cli" => Domain::Environment, // environment + args live under cli
        "clocks" => Domain::Clock,
        "sockets" => Domain::Network,
        "random" => Domain::Random,
        "io" => Domain::Stdio,
        _ => return None,
    })
}

/// Extract the capability requirements implied by a parsed module's imports.
pub fn requirements_from_module(module: &Module) -> Vec<Requirement> {
    let mut out = Vec::new();
    for import in &module.imports {
        // Only function imports carry behavior; memories/tables/globals are
        // data plumbing and do not by themselves grant host capabilities.
        if import.kind != ExternalKind::Func {
            continue;
        }
        let domain = if import.module.starts_with("wasi_snapshot_preview1")
            || import.module == "wasi_unstable"
        {
            classify_preview1(&import.field).unwrap_or(Domain::Unknown)
        } else if import.module.starts_with("wasi:") {
            classify_interface(&import.module).unwrap_or(Domain::Unknown)
        } else {
            Domain::Unknown
        };
        out.push(Requirement {
            domain,
            source: import.module.clone(),
            detail: import.field.clone(),
        });
    }
    out
}

/// Extract capability requirements from a WIT-like manifest's imports.
pub fn requirements_from_manifest(manifest: &Manifest) -> Vec<Requirement> {
    let mut out = Vec::new();
    for path in manifest.all_imports() {
        let domain = classify_interface(&path).unwrap_or(Domain::Unknown);
        out.push(Requirement {
            domain,
            source: path.clone(),
            detail: path,
        });
    }
    out
}

/// A rule for one capability domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainRule {
    /// Whether the domain is permitted at all.
    pub allow: bool,
    /// Optional allow-list of resource tokens (e.g. path prefixes, hosts).
    /// When empty and `allow` is true, all resources in the domain are allowed.
    pub allow_list: Vec<String>,
}

/// An explicit capability policy: default stance plus per-domain rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Policy name, for reports.
    pub name: String,
    /// The stance applied to any domain without an explicit rule.
    pub default_allow: bool,
    /// Explicit per-domain rules that override the default stance.
    pub rules: BTreeMap<Domain, DomainRule>,
}

impl Default for Policy {
    fn default() -> Self {
        // Deny-by-default is the safe baseline.
        Policy {
            name: "default-deny".into(),
            default_allow: false,
            rules: BTreeMap::new(),
        }
    }
}

impl Policy {
    /// Resolve the effective rule for a domain, honoring the default stance.
    pub fn effective(&self, domain: Domain) -> DomainRule {
        if let Some(rule) = self.rules.get(&domain) {
            rule.clone()
        } else {
            DomainRule {
                allow: self.default_allow,
                allow_list: Vec::new(),
            }
        }
    }

    /// Parse a policy file. The format is documented in `docs/FORMAT.md`:
    ///
    /// ```text
    /// policy "sandbox-strict"
    /// default deny
    /// allow clock
    /// allow fs: /tmp, /var/cache
    /// deny network
    /// ```
    pub fn parse(text: &str) -> Result<Policy> {
        let mut policy = Policy::default();
        for (lineno, raw) in text.lines().enumerate() {
            let line = match raw.find('#') {
                Some(i) => &raw[..i],
                None => raw,
            }
            .trim();
            if line.is_empty() {
                continue;
            }

            if let Some(rest) = line.strip_prefix("policy ") {
                policy.name = rest.trim().trim_matches('"').to_string();
                continue;
            }
            if let Some(rest) = line.strip_prefix("default ") {
                policy.default_allow = match rest.trim() {
                    "allow" => true,
                    "deny" => false,
                    other => {
                        return Err(Error::new(
                            ErrorKind::BadManifest,
                            format!(
                                "line {}: default must be allow|deny, got '{}'",
                                lineno + 1,
                                other
                            ),
                        ))
                    }
                };
                continue;
            }

            let (allow, body) = if let Some(r) = line.strip_prefix("allow ") {
                (true, r)
            } else if let Some(r) = line.strip_prefix("deny ") {
                (false, r)
            } else {
                return Err(Error::new(
                    ErrorKind::BadManifest,
                    format!(
                        "line {}: unrecognized policy directive '{}'",
                        lineno + 1,
                        line
                    ),
                ));
            };

            let (domain_str, list) = match body.split_once(':') {
                Some((d, l)) => (d.trim(), parse_list(l)),
                None => (body.trim(), Vec::new()),
            };
            let domain = Domain::from_slug(domain_str).ok_or_else(|| {
                Error::new(
                    ErrorKind::BadManifest,
                    format!(
                        "line {}: unknown capability domain '{}'",
                        lineno + 1,
                        domain_str
                    ),
                )
            })?;
            policy.rules.insert(
                domain,
                DomainRule {
                    allow,
                    allow_list: list,
                },
            );
        }
        Ok(policy)
    }
}

fn parse_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

/// The verdict for a single domain after evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainVerdict {
    /// The capability domain this verdict concerns.
    pub domain: Domain,
    /// Whether the component actually requires this domain.
    pub required: bool,
    /// Whether the policy permits it.
    pub allowed: bool,
    /// The specific requirements that fed this verdict.
    pub requirements: Vec<Requirement>,
}

impl DomainVerdict {
    /// A violation occurs when the domain is required but not allowed.
    pub fn is_violation(&self) -> bool {
        self.required && !self.allowed
    }
}

/// The full outcome of evaluating a component's requirements against a policy.
#[derive(Debug, Clone)]
pub struct Evaluation {
    /// The name of the policy that was evaluated.
    pub policy_name: String,
    /// One verdict per capability domain.
    pub verdicts: Vec<DomainVerdict>,
}

impl Evaluation {
    /// True when no required domain is denied.
    pub fn is_compliant(&self) -> bool {
        !self.verdicts.iter().any(|v| v.is_violation())
    }

    /// The list of domains that violate the policy.
    pub fn violations(&self) -> Vec<Domain> {
        self.verdicts
            .iter()
            .filter(|v| v.is_violation())
            .map(|v| v.domain)
            .collect()
    }
}

/// Evaluate a set of requirements against a policy.
pub fn evaluate(requirements: &[Requirement], policy: &Policy) -> Evaluation {
    // Group requirements by domain.
    let mut by_domain: BTreeMap<Domain, Vec<Requirement>> = BTreeMap::new();
    for req in requirements {
        by_domain.entry(req.domain).or_default().push(req.clone());
    }

    let mut verdicts = Vec::new();
    // Evaluate every known domain plus Unknown so the report is exhaustive.
    let mut domains: Vec<Domain> = Domain::ALL.to_vec();
    domains.push(Domain::Unknown);

    for domain in domains {
        let reqs = by_domain.get(&domain).cloned().unwrap_or_default();
        let required = !reqs.is_empty();
        let rule = policy.effective(domain);
        // Unknown imports are only "allowed" if the policy is explicitly
        // permissive for the unknown bucket; otherwise they always violate.
        verdicts.push(DomainVerdict {
            domain,
            required,
            allowed: rule.allow,
            requirements: reqs,
        });
    }

    Evaluation {
        policy_name: policy.name.clone(),
        verdicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::Import;

    fn func_import(module: &str, field: &str) -> Import {
        Import {
            module: module.into(),
            field: field.into(),
            kind: ExternalKind::Func,
        }
    }

    #[test]
    fn classifies_preview1() {
        assert_eq!(classify_preview1("fd_write"), Some(Domain::Filesystem));
        assert_eq!(classify_preview1("path_open"), Some(Domain::Filesystem));
        assert_eq!(classify_preview1("environ_get"), Some(Domain::Environment));
        assert_eq!(classify_preview1("clock_time_get"), Some(Domain::Clock));
        assert_eq!(classify_preview1("sock_recv"), Some(Domain::Network));
        assert_eq!(classify_preview1("random_get"), Some(Domain::Random));
        assert_eq!(classify_preview1("proc_exit"), None);
    }

    #[test]
    fn classifies_interfaces() {
        assert_eq!(
            classify_interface("wasi:filesystem/types"),
            Some(Domain::Filesystem)
        );
        assert_eq!(
            classify_interface("wasi:sockets/tcp"),
            Some(Domain::Network)
        );
        assert_eq!(
            classify_interface("wasi:clocks/wall-clock@0.2.0"),
            Some(Domain::Clock)
        );
        assert_eq!(classify_interface("acme:custom/thing"), None);
    }

    #[test]
    fn requirements_skip_non_func_imports() {
        let mut m = Module::default();
        m.imports
            .push(func_import("wasi_snapshot_preview1", "clock_time_get"));
        m.imports.push(Import {
            module: "env".into(),
            field: "memory".into(),
            kind: ExternalKind::Memory,
        });
        let reqs = requirements_from_module(&m);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].domain, Domain::Clock);
    }

    #[test]
    fn policy_parse_and_evaluate() {
        let policy =
            Policy::parse("policy \"strict\"\ndefault deny\nallow clock\nallow fs: /tmp\n")
                .unwrap();
        assert_eq!(policy.name, "strict");
        assert!(!policy.default_allow);
        assert!(policy.effective(Domain::Clock).allow);
        assert_eq!(
            policy.effective(Domain::Filesystem).allow_list,
            vec!["/tmp".to_string()]
        );

        let reqs = vec![
            Requirement {
                domain: Domain::Clock,
                source: "w".into(),
                detail: "clock_time_get".into(),
            },
            Requirement {
                domain: Domain::Network,
                source: "w".into(),
                detail: "sock_recv".into(),
            },
        ];
        let eval = evaluate(&reqs, &policy);
        assert!(!eval.is_compliant());
        assert_eq!(eval.violations(), vec![Domain::Network]);
    }

    #[test]
    fn default_allow_permits_everything() {
        let policy = Policy::parse("policy \"open\"\ndefault allow\n").unwrap();
        let reqs = vec![Requirement {
            domain: Domain::Network,
            source: "w".into(),
            detail: "sock_recv".into(),
        }];
        assert!(evaluate(&reqs, &policy).is_compliant());
    }

    #[test]
    fn unknown_import_violates_default_deny() {
        let policy = Policy::default();
        let mut m = Module::default();
        m.imports.push(func_import("shady_host", "do_stuff"));
        let reqs = requirements_from_module(&m);
        let eval = evaluate(&reqs, &policy);
        assert!(eval.violations().contains(&Domain::Unknown));
    }
}
