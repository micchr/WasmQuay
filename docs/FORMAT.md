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

