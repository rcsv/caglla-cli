.PHONY: check test run clean

check:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test
	cargo build

test:
	cargo test

run:
	cargo run

clean:
	cargo clean
