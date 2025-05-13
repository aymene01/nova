# Makefile for Rust CI project

.PHONY: all format lint test build

# Run all checks
all: format lint test build

# Check formatting using rustfmt
format:
	cargo fmt -- --check

# Lint the code with clippy, treating warnings as errors
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
test:
	cargo test --all

# Build the project in release mode
build:
	cargo build --release
