# Examples

Ready-to-run inputs and captured outputs for the wasmquay toolkit. Every file
here was produced by the tools in this repository — nothing is hand-edited.

## Inputs

| File                   | What it is                                                        |
|------------------------|-------------------------------------------------------------------|
| `sandbox-strict.pol`   | Deny-by-default policy; allows clock, stdio and scoped fs.        |
| `edge-networked.pol`   | Allows clock, stdio and network; denies filesystem.               |
| `image-pipeline.wit`   | A WIT-like manifest with two worlds.                              |

The binary modules live in `../fixtures/` and are regenerated with
`wasmquay gen-fixtures`.

## Captured reports (JSON)

| File                     | Command that produced it                                          |
|--------------------------|-------------------------------------------------------------------|
| `clock-inspection.json`  | `wasmquay inspect fixtures/clock-service.wasm --pretty`           |
| `net-inspection.json`    | `wasmquay inspect fixtures/net-service.wasm --pretty`             |
| `net-policy.json`        | `wasmquay policy fixtures/net-service.wasm sandbox-strict.pol --pretty` |
| `compat-clock-net.json`  | `wasmquay compat fixtures/clock-service.wasm fixtures/net-service.wasm --pretty` |

## Try it

```sh
# 1. Build the CLI and (re)generate the fixtures.
cargo build --release
./target/release/wasmquay gen-fixtures fixtures

# 2. Evaluate the networked service against the strict sandbox policy.
#    Exits 3 because it needs the network, which the policy denies.
./target/release/wasmquay policy fixtures/net-service.wasm examples/sandbox-strict.pol

# 3. Feed a JSON report into the TypeScript explorer for a risk view.
cd explorer && npm run build && cd ..
./target/release/wasmquay inspect fixtures/net-service.wasm --json > net.json
./target/release/wasmquay policy  fixtures/net-service.wasm examples/sandbox-strict.pol --json > pol.json
node explorer/dist/cli.js surface net.json pol.json
```
