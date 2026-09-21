# wasmquay — build orchestration
#
# Targets are POSIX-make friendly and mirror what CI runs. The Rust side is
# a Cargo workspace; the TypeScript explorer is a self-contained npm package.

CARGO ?= cargo
NPM   ?= npm
FIXTURES_DIR ?= fixtures

.PHONY: all build test rust-build rust-test rust-fmt rust-clippy \
        ts-build ts-test fixtures examples clean help

all: build test ## Build and test everything

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN{FS=":.*?## "}{printf "  %-14s %s\n", $$1, $$2}'

build: rust-build ts-build ## Build Rust workspace and TS explorer

test: rust-test ts-test ## Run all tests

## --- Rust -------------------------------------------------------------

rust-build: ## Build the Rust workspace (release)
	$(CARGO) build --release

rust-test: ## Run Rust unit + integration + doc tests
	$(CARGO) test

rust-fmt: ## Check Rust formatting
	$(CARGO) fmt --all -- --check

rust-clippy: ## Lint with clippy (deny warnings)
	$(CARGO) clippy --all-targets -- -D warnings

## --- TypeScript -------------------------------------------------------

ts-build: ## Compile the TypeScript explorer
	cd explorer && $(NPM) run build

ts-test: ## Type-check and run the explorer tests (offline, no install)
	cd explorer && $(NPM) test

## --- Fixtures & examples ---------------------------------------------

fixtures: rust-build ## (Re)generate the binary .wasm fixtures
	./target/release/wasmquay gen-fixtures $(FIXTURES_DIR)

examples: rust-build ts-build ## Produce example JSON reports and analyze them
	./target/release/wasmquay inspect $(FIXTURES_DIR)/net-service.wasm --pretty > examples/net-inspection.json
	./target/release/wasmquay policy  $(FIXTURES_DIR)/net-service.wasm examples/sandbox-strict.pol --pretty > examples/net-policy.json
	node explorer/dist/cli.js surface examples/net-inspection.json examples/net-policy.json || true

clean: ## Remove build artifacts
	$(CARGO) clean
	rm -rf explorer/dist explorer/dist-test
