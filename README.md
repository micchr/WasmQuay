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
