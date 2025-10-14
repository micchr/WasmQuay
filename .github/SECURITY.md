# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 1.x     | Yes       |
| 0.x     | No        |

## Reporting a vulnerability

wasmquay only reads WebAssembly bytes and text files; it never executes the
modules it inspects and makes no network calls. If you find a security issue -
for example a panic or excessive allocation on a hostile binary, a path
traversal when writing fixtures, or a report that leaks more of the input than
it claims to - please report it privately with GitHub's "Report a
vulnerability" button on the Security tab.

We aim to acknowledge reports within 72 hours and to ship a fix or mitigation
in the next patch release. Please do not open a public issue before a fix is
available.
