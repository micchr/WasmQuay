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
