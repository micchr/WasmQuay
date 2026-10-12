# Capability lattice (sketch)

A review aid: arrange the seven domains on a lattice ordered by blast radius -
`network` / `unknown` at the top, then `fs`, `env`, and `clock` / `random` /
`stdio` beneath - and plot every inspected component against it.

Each node would read its data straight from the `wasmquay/*@1` reports: domain
requirements come from the inspection JSON, policy verdicts from the policy
JSON. A component that widens the capability set between two versions (per
`compat`) would get a highlighted edge from the old surface to the new one.

Nothing here requires new backend code: it is a renderer experiment on top of
the existing report schemas, likely a new view in `wasmquay-explore`.
