# wasmquay — build orchestration
#
# Targets are POSIX-make friendly and mirror what CI runs. The Rust side is
# a Cargo workspace; the TypeScript explorer is a self-contained npm package.

CARGO ?= cargo
NPM   ?= npm
FIXTURES_DIR ?= fixtures

.PHONY: all build test rust-build rust-test rust-fmt rust-clippy \
        ts-build ts-test fixtures examples clean help

