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
