<!-- ░░░ wasmquay ░░░ -->
<div align="center">

<img src="docs/assets/quay-hero.svg" alt="wasmquay — offline WebAssembly component inspection and capability policy" width="100%"/>

```text
╔══════════════════════════════════════════════════════════════════════╗
║  w a s m q u a y  ·  the dry-dock for WebAssembly components           ║
║  inspect · classify · gate · compare — 100% offline, 0 dependencies    ║
╚══════════════════════════════════════════════════════════════════════╝
```

**A mixed-language toolkit that decodes real `.wasm` binaries, classifies the
capabilities they demand from the host, and gates them against explicit
policies — then hands a typed JSON manifest to a static TypeScript explorer.**

`rust · std-only` &nbsp;•&nbsp; `typescript · tsc-only` &nbsp;•&nbsp; `no network` &nbsp;•&nbsp; `no runtime magic`

</div>

---

## `0x00` — TL;DR for the impatient

```console
$ cargo build --release                         # zero third-party crates
$ ./target/release/wasmquay gen-fixtures fixtures
wrote fixtures/clock-service.wasm (139 bytes)
wrote fixtures/net-service.wasm (208 bytes)
wrote fixtures/fs-component.wasm (135 bytes)
wrote fixtures/opaque-host.wasm (69 bytes)

$ ./target/release/wasmquay inspect fixtures/net-service.wasm
wasmquay inspection :: net-service.wasm
  version=1  bytes=208  sections=3
  module-name: net-service
  imports=5 (funcs=4, memories=1)  exports=1
  capability domains:
    - fs       x1
    - clock    x1
    - network  x2

$ ./target/release/wasmquay policy fixtures/net-service.wasm examples/sandbox-strict.pol
policy 'sandbox-strict': VIOLATION
  [allow] fs       (1 reqs)
  [allow] clock    (1 reqs)
  [DENY ] network  (2 reqs)
$ echo $?
3
```

That last exit code is the point: **wasmquay is designed to gate CI**. A
component that reaches for a capability its policy forbids fails the build.

---

## `0x01` — What this actually is (and is not)

wasmquay is a **static analyzer** for the WebAssembly ecosystem. It answers one
question well: *what can this component do to the outside world, and is that
allowed?*

A WebAssembly module is a sealed box. The only way it can touch a filesystem,
open a socket, read the clock, or learn the time of day is through the host
functions it **imports**. wasmquay decodes those imports straight from the
binary, buckets each one into a capability **domain**, and checks the result
against a policy you write in plain text.

> ### ⚠️ Scope & honesty
> wasmquay **does not execute, instantiate, or sandbox** WebAssembly. It is not
> a runtime and makes no runtime-enforcement claims. When the docs say
> "enforce", they mean *analyze against a stated policy at inspection time* —
> the kind of check you run in CI, not an interception layer around a live
> module. Everything it reports is derived from bytes on disk. That honesty is a
> feature: the analysis is deterministic, offline, and auditable.

What it genuinely does, verified by its own test suite:

- **Decodes** the WebAssembly core binary format: the `\0asm` header, the
  section table (ids/offsets/sizes), the import and export sections, and the
  custom `name` section (module + function names).
- **Classifies** every host import into `fs · env · clock · network · random ·
  stdio · unknown`, covering both WASI preview1 function names and preview2 /
  component-model interface paths.
- **Evaluates** a component's capability set against an explicit, deny-by-default
  policy with per-domain allow-lists.
- **Compares** two components for substitutability (is B a safe drop-in for A?).
- **Reads** a documented WIT-like interface manifest subset.
- **Emits** stable, versioned JSON and human text — consumed by a companion
  **TypeScript explorer** that computes risk scores and reconciles findings.

---

## `0x02` — Architecture

```text
        ┌──────────────────────── RUST (std-only) ─────────────────────────┐
        │                                                                   │
  .wasm │   leb ──▶ wasm ──▶ policy ──▶ compat                              │
  ──────┼─▶ ┌─────┐  ┌────┐  ┌──────┐  ┌──────┐        ┌──────────┐         │
  .wit  │   │bytes│  │decode│ │classify││diff │  ──────▶│  report  │─┐       │
  ──────┼─▶ │+LEB │  │hdr/  │ │fs/env/ ││expo │        │ json/txt │ │       │
  .pol  │   │     │  │sect/ │ │clock/  ││caps │        └──────────┘ │       │
  ──────┼─▶ └─────┘  │imp/  │ │net/... │└──────┘             ▲       │       │
        │            │exp/  │ └──────┘                       │       │       │
        │            │name  │              fixture ──────────┘       │       │
        │            └──────┘              (binary encoder)          │       │
        └────────────────────────────────────────────────────────────┼──────┘
                                                                       │ JSON
        ┌──────────────────── TYPESCRIPT (tsc-only) ───────────────────▼──────┐
        │   parse (schema guards) ──▶ analyze (risk + reconcile) ──▶ render    │
        │                          wasmquay-explore CLI                        │
        └───────────────────────────────────────────────────────────────────┘
```

| Crate / package        | Role                                                             |
|------------------------|------------------------------------------------------------------|
| `wasmquay-core`        | The engine: decoding, classification, policy, compat, JSON.      |
| `wasmquay-cli`         | The `wasmquay` binary and its subcommands.                       |
| `wasmquay-explorer`    | TypeScript library + `wasmquay-explore` CLI over the JSON reports.|

Both language halves are **dependency-free at runtime**. The Rust workspace
pulls in *zero* third-party crates; the TypeScript package needs only the
compiler itself (it ships its own minimal ambient type shims so it type-checks
without downloading `@types/node`).

<div align="center">
<img src="docs/assets/capability-crane.svg" alt="capability crane sorting fs/env/clock/network containers into allow and deny bins" width="70%"/>
</div>

---

## `0x03` — Install & build

Requirements: a stable Rust toolchain (≥ 1.74) and Node.js (≥ 20). Nothing else.

```console
# Rust workspace — the whole thing compiles offline.
$ cargo build --release
$ cargo test                    # 43 tests: unit + integration + doctest

# TypeScript explorer.
$ cd explorer
$ npm run build                 # tsc -> dist/
$ npm test                      # tsc -> dist-test/ + node --test  (12 tests)
```

Or drive both sides through the `Makefile`:

```console
$ make build      # rust-build + ts-build
$ make test       # rust-test  + ts-test
$ make fixtures   # regenerate the .wasm fixtures
$ make examples   # produce example JSON reports and analyze them
$ make help       # list every target
```

---

## `0x04` — The command surface

```text
wasmquay <COMMAND> [ARGS] [--json] [--pretty]

  inspect <file.wasm>                 decode header/sections/imports/exports
  caps <file.wasm>                    list classified capability requirements
  policy <file.wasm> <policy.pol>     evaluate a module against a policy
  compat <baseline.wasm> <cand.wasm>  compare two modules for compatibility
  manifest <file.wit>                 parse a WIT-like interface manifest
  gen-fixtures <dir>                  write the bundled .wasm fixtures
```

Exit codes are CI-shaped: `0` ok/compliant/compatible · `1` usage · `2`
parse/IO · `3` policy violation or incompatibility.

### `inspect` — read the binary

```console
$ ./target/release/wasmquay inspect fixtures/clock-service.wasm --pretty
{
  "schema": "wasmquay/inspection@1",
  "source": "clock-service.wasm",
  "header": { "magic": "\\0asm", "version": 1, "byte_length": 139, "module_name": "clock-service" },
  "sections": [
    { "id": 2, "name": "import", "custom_name": null, "offset": 10, "size": 89 },
    { "id": 7, "name": "export", "custom_name": null, "offset": 101, "size": 7 },
    { "id": 0, "name": "custom", "custom_name": "name", "offset": 110, "size": 29 }
  ],
  "imports": [
    { "module": "wasi_snapshot_preview1", "field": "clock_time_get", "kind": "func" },
    { "module": "wasi_snapshot_preview1", "field": "fd_write", "kind": "func" },
    { "module": "env", "field": "memory", "kind": "memory" }
  ],
  "exports": [ { "field": "run", "kind": "func", "index": 0 } ],
  "names": [ { "index": 0, "name": "run" } ],
  "capabilities": [
    { "domain": "fs",    "requirements": [ { "source": "wasi_snapshot_preview1", "detail": "fd_write" } ] },
    { "domain": "clock", "requirements": [ { "source": "wasi_snapshot_preview1", "detail": "clock_time_get" } ] }
  ]
}
```

Every offset and size above is read from the actual bytes — you can seek to
`offset` in the file and find exactly that section payload.

### `caps` — just the capability surface

```console
$ ./target/release/wasmquay caps fixtures/fs-component.wasm
fs       wasi:filesystem/types :: read-via-stream
clock    wasi:clocks/wall-clock :: now
```

Note the different import style: `fs-component.wasm` imports **component-model
interface paths** (`wasi:filesystem/types`) rather than preview1 function
names — wasmquay classifies both.

### `policy` — gate it

```console
$ cat examples/sandbox-strict.pol
policy "sandbox-strict"
default deny
allow clock
allow stdio
allow fs: /tmp, /var/cache/worker
deny network
deny env

$ ./target/release/wasmquay policy fixtures/clock-service.wasm examples/sandbox-strict.pol
policy 'sandbox-strict': COMPLIANT
  [allow] fs       (1 reqs)
  [allow] clock    (1 reqs)

$ ./target/release/wasmquay policy fixtures/net-service.wasm examples/sandbox-strict.pol ; echo "exit=$?"
policy 'sandbox-strict': VIOLATION
  [allow] fs       (1 reqs)
  [allow] clock    (1 reqs)
  [DENY ] network  (2 reqs)
exit=3
```

### `compat` — is it a safe swap?

```console
$ ./target/release/wasmquay compat fixtures/clock-service.wasm fixtures/net-service.wasm
compat clock-service.wasm <- net-service.wasm: BREAKING
  ! new capability required: network via wasi_snapshot_preview1
```

`net-service` keeps every export `clock-service` had, so exports are fine — but
it *adds* a network requirement. A host that safely ran the clock service might
not be prepared to grant sockets, so the swap is flagged **BREAKING**.

The reverse direction is compatible (dropping a capability never breaks a host):

```console
$ ./target/release/wasmquay compat fixtures/net-service.wasm fixtures/clock-service.wasm
compat net-service.wasm <- clock-service.wasm: COMPATIBLE
```

### `manifest` — read a WIT-like world

```console
$ ./target/release/wasmquay manifest examples/image-pipeline.wit
package: acme:image-pipeline@1.4.0
world processor (3 imports, 2 exports)
world thumbnailer (1 imports, 1 exports)
classified capability domains:
  clock    wasi:clocks/wall-clock
  fs       wasi:filesystem/types
  stdio    wasi:io/streams
```

---

## `0x05` — The TypeScript explorer

The Rust CLI produces JSON; the explorer turns it into an opinionated risk view.
It executes nothing — it is a pure function of the report you feed it.

```console
$ ./target/release/wasmquay inspect fixtures/net-service.wasm --json > net.json
$ ./target/release/wasmquay policy fixtures/net-service.wasm examples/sandbox-strict.pol --json > pol.json

$ node explorer/dist/cli.js surface net.json pol.json
▚ net-service.wasm (net-service)
  risk: ● HIGH (score 22.9)
  imports=5 exports=1
  capabilities:
    • network  x2  — can open sockets / exfiltrate data
    • fs       x1  — can read or modify files
    • clock    x1  — can read wall/monotonic clocks

policy 'sandbox-strict': VIOLATION
  ✓ [ok] 'fs' required and allowed
  ✓ [ok] 'clock' required and allowed
  ✗ [violation] 'network' is required but denied by policy 'sandbox-strict' — can open sockets / exfiltrate data
```

Rank several components by static risk:

```console
$ node explorer/dist/cli.js rank net.json clock.json
capability risk ranking (highest first):
  1. net-service.wasm         HIGH      score=22.9
  2. clock-service.wasm       ELEVATED  score=10
```

The risk score weights domains by blast radius (`network`/`unknown` = 5,
`fs` = 4, `env` = 2, `clock`/`random`/`stdio` = 1) and scales sub-linearly with
how many distinct entry points a domain has, so one chatty domain can't drown
out the signal. The exact weights and bands live in
[`explorer/src/analyze.ts`](explorer/src/analyze.ts).

Use it as a library, too:

```ts
import { parseInspection, analyzeSurface } from "wasmquay-explorer";

const surface = analyzeSurface(parseInspection(reportJson));
if (surface.hasUnknown) {
  throw new Error(`${surface.source} imports an unclassified host!`);
}
console.log(`${surface.source}: ${surface.band} (score ${surface.score})`);
```

---

## `0x06` — Capability classification, at a glance

| Import shape                                   | Domain     | Rationale                        |
|------------------------------------------------|------------|----------------------------------|
| `preview1` `fd_*`, `path_*`                    | `fs`       | file & directory I/O             |
| `preview1` `environ_*`, `args_*`               | `env`      | ambient environment / argv       |
| `preview1` `clock_*`                           | `clock`    | wall / monotonic time            |
| `preview1` `sock_*`                            | `network`  | sockets                          |
| `preview1` `random_get`                        | `random`   | randomness source                |
| `wasi:filesystem/*`                            | `fs`       | component-model filesystem       |
| `wasi:cli/*`                                   | `env`      | environment + args               |
| `wasi:clocks/*`                                | `clock`    | component-model clocks           |
| `wasi:sockets/*`                               | `network`  | component-model sockets          |
| `wasi:random/*`                                | `random`   | component-model randomness       |
| `wasi:io/*`                                    | `stdio`    | streams                          |
| *anything else*                                | `unknown`  | **unrecognized host — untrusted**|

Memory/table/global imports are data plumbing and are **not** treated as
capabilities. An `unknown` import always violates a deny-by-default policy —
wasmquay never silently ignores a host dependency it can't name.

The full grammar for every format (WIT-like manifest, `.pol` policy, and all
four JSON schemas) is specified in **[`docs/FORMAT.md`](docs/FORMAT.md)**.

---

## `0x07` — Fixtures

The repository ships four deterministic binary fixtures under `fixtures/`, and
the exact encoder that produced them (`wasmquay-core::fixture`). Regenerate them
byte-for-byte any time:

```console
$ ./target/release/wasmquay gen-fixtures fixtures
```

Because the encoder writes the same grammar the decoder reads, the round-trip
`build → parse` is itself an end-to-end test of the format code
(`fixture::tests::round_trips_through_parser`).

| Fixture                | Shape                                                          |
|------------------------|----------------------------------------------------------------|
| `clock-service.wasm`   | clock + stdio + fs (well-behaved worker)                       |
| `net-service.wasm`     | clock service **plus** sockets (networked variant)             |
| `fs-component.wasm`    | component-model preview2 interface imports                     |
| `opaque-host.wasm`     | imports an unrecognized host → classified `unknown`            |

---

## `0x08` — Testing & CI

```console
$ cargo test
    test result: ok. 4 passed;  0 failed;   # wasmquay-cli
    test result: ok. 38 passed; 0 failed;   # wasmquay-core
    test result: ok. 1 passed;  0 failed;   # doctest

$ cd explorer && npm test
    ℹ tests 12
    ℹ pass 12
    ℹ fail 0
```
