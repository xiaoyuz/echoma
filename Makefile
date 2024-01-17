.PHONY: check-all test-all fmt-all release

check-all:
	cargo check --all --all-targets --all-features
	cargo fmt -- --check
	cargo clippy --all-targets --all-features -- -D clippy::all

fmt-all:
	cargo clippy --fix --allow-dirty --allow-staged
	cargo fmt

test-all:
	cargo test --all

release:
	cargo build --release