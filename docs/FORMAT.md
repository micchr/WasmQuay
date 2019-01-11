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
