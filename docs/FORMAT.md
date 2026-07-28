# wasmquay Formats

This document is the normative reference for every textual and JSON format the
toolkit reads or writes. It is intended to be precise enough to implement an
independent consumer.

---

## 1. WebAssembly binary subset (decoded, not defined here)

`wasmquay-core` decodes standard WebAssembly **core** binary modules. It does
not redefine that format; it follows the WebAssembly core specification. The
decoder reads:

| Structure            | Bytes                                                       |
|----------------------|-------------------------------------------------------------|
| Header magic         | `00 61 73 6D` (`\0asm`)                                     |
| Header version       | little-endian `u32` (must be non-zero)                      |
| Section              | `id:u8` `size:uleb128` `payload[size]`                      |
| Import               | `module:name` `field:name` `kind:u8` `desc`                 |
| Export               | `field:name` `kind:u8` `index:uleb128`                      |
| `name` custom section| subsection `id:u8` `size:uleb128`; `0`=module, `1`=funcs    |

`name` is a length-prefixed UTF-8 string: `len:uleb128` followed by `len` bytes.
Integers use LEB128 (unsigned or signed as the grammar dictates).

External `kind` bytes: `0x00` func, `0x01` table, `0x02` memory, `0x03` global.
Any other byte is preserved as `other:0xNN`.

**Scope note.** The decoder walks *all* sections structurally (id, offset,
size) and deep-decodes the import, export and `name` sections — the pieces that
determine a component's capability surface. It does not decode the code section
body or component-model type definitions.

---

## 2. WIT-like interface manifest (`.wit`)

A deliberately small, line-oriented subset that captures interface imports and
exports. It is **not** full WIT.

### Grammar

```
manifest    := (line)*
line        := blank | comment | package | world-open | world-close | decl
comment     := "//" .*                      ; also allowed at end of line
package     := "package" WS text
world-open  := "world" WS name ("{")?
world-close := "}"
decl        := ("import" | "export") WS iref
iref        := path (": " signature)?        ; ": " = colon + whitespace
```

* Blank lines and `//` comments are ignored anywhere.
* A `:` that is **immediately followed by whitespace** introduces a signature
  (e.g. `export process: func(...)`). A `:` inside a path
  (`wasi:filesystem/types`) is part of the path.
* A missing closing `}` at end of file is tolerated.
* `import`/`export` outside a `world` is an error.

### Example

```
package acme:image-pipeline@1.4.0

world processor {
    import wasi:filesystem/types
    import wasi:clocks/wall-clock
    export process: func(input: list<u8>) -> list<u8>
}
```

---

## 3. Capability policy (`.pol`)

A line-oriented policy describing which capability domains a component may use.

### Grammar

```
policy-file := (line)*
line        := blank | comment | name | default | rule
comment     := "#" .*                        ; also allowed at end of line
name        := "policy" WS text              ; quotes optional, stripped
default     := "default" WS ("allow" | "deny")
rule        := ("allow" | "deny") WS domain (":" list)?
domain      := "fs" | "env" | "clock" | "network" | "random" | "stdio" | "unknown"
list        := token ("," token)*            ; resource allow-list
```

* `default` sets the stance for any domain without an explicit rule. If absent,
  the default is **deny**.
* `network` and `net` are accepted aliases, as are `random`/`rand`.
* The resource `list` (e.g. path prefixes, host names) is recorded on the rule
  for auditing. The evaluator currently gates on the domain allow/deny bit;
  resource-level matching is exposed in the parsed model for downstream tools.

### Capability classification

Imports are classified into domains as follows:

| Import source                          | Domain     |
|----------------------------------------|------------|
| `wasi_snapshot_preview1` `fd_*`,`path_*`| `fs`       |
| `..._preview1` `environ_*`,`args_*`     | `env`      |
| `..._preview1` `clock_*`                | `clock`    |
| `..._preview1` `sock_*`                 | `network`  |
| `..._preview1` `random_get`             | `random`   |
| `wasi:filesystem/*`                     | `fs`       |
| `wasi:cli/*`                            | `env`      |
| `wasi:clocks/*`                         | `clock`    |
| `wasi:sockets/*`                        | `network`  |
| `wasi:random/*`                         | `random`   |
| `wasi:io/*`                             | `stdio`    |
| anything else                           | `unknown`  |

Non-function imports (memories, tables, globals) do not by themselves grant a
host capability and are excluded from classification.

---

## 4. JSON reports

All reports are UTF-8 JSON objects with a `schema` discriminator. Object key
order is stable across runs.

### 4.1 `wasmquay/inspection@1`

```json
{
  "schema": "wasmquay/inspection@1",
  "source": "net-service.wasm",
  "header": { "magic": "\\0asm", "version": 1, "byte_length": 208, "module_name": "net-service" },
  "sections": [ { "id": 2, "name": "import", "custom_name": null, "offset": 10, "size": 90 } ],
  "imports":  [ { "module": "wasi_snapshot_preview1", "field": "sock_recv", "kind": "func" } ],
  "exports":  [ { "field": "run", "kind": "func", "index": 0 } ],
  "names":    [ { "index": 0, "name": "run" } ],
  "capabilities": [
    { "domain": "network", "requirements": [ { "source": "wasi_snapshot_preview1", "detail": "sock_recv" } ] }
  ]
}
```

`module_name` and `custom_name` are `null` when absent.

### 4.2 `wasmquay/policy@1`

```json
{
  "schema": "wasmquay/policy@1",
  "policy": "sandbox-strict",
  "compliant": false,
  "violations": ["network"],
  "verdicts": [
    { "domain": "network", "required": true, "allowed": false, "violation": true,
      "requirements": ["wasi_snapshot_preview1 :: sock_recv"] }
  ]
}
```

A `verdict` exists for every domain; `required` marks the ones the component
actually uses. `violation == required && !allowed`.

### 4.3 `wasmquay/compat@1`

```json
{
  "schema": "wasmquay/compat@1",
  "baseline": "clock-service.wasm",
  "candidate": "net-service.wasm",
  "compatible": false,
  "breaking_reasons": ["new capability required: network via wasi_snapshot_preview1"],
  "export_diffs": [ { "change": "added", "name": "extra", "kind": "func" } ],
  "capability_diffs": [ { "change": "added", "domain": "network", "source": "wasi_snapshot_preview1" } ]
}
```

Compatibility rule: the candidate is a safe drop-in when **no export is
removed** and **no new capability domain/source is added**.

### 4.4 `wasmquay/manifest@1`

Emitted by `wasmquay manifest --json`.

```json
{
  "schema": "wasmquay/manifest@1",
  "package": "acme:image-pipeline@1.4.0",
  "worlds": [ { "name": "processor", "imports": ["wasi:filesystem/types"], "exports": ["process"] } ],
  "capabilities": [ { "domain": "fs", "source": "wasi:filesystem/types" } ]
}
```

---

## 5. Exit codes

Both CLIs share a convention so they compose in CI:

| Code | Meaning                                   |
|------|-------------------------------------------|
| 0    | success / compliant / compatible          |
| 1    | usage error                               |
| 2    | parse or I/O error                        |
| 3    | policy violation or incompatibility       |
