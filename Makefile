.PHONY: lint
lint:
	cargo clippy --all-targets --all-features

.PHONY: build
build:
	cargo build --all-targets --all-features

.PHONY: lint_fix
lint_fix:
	cargo clippy --all-targets --all-features --fix --allow-dirty

.PHONY: format
format:
	cargo fmt --all

.PHONY: test
test:
	cargo test --all-targets --all-features  -- --nocapture ${NAME}

.PHONY: coverage
coverage:
	cargo install cargo-tarpaulin
	cargo tarpaulin --fail-under 80 --all-targets --all-features --locked --target-dir target/coverage -- --nocapture

.PHONY: test_ci
test_ci:
	cargo test --all-targets --all-features  -- --nocapture 