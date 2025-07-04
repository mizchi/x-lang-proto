# EffectLang Makefile

.PHONY: all build test bench clean lint fmt check examples help

# Default target
all: build test

# Build the project
build:
	cargo build --release

# Build only CLI (faster for development)
build-cli:
	cargo build --bin effect-cli --release

# Run all tests
test:
	cargo test

# Run specific test suites
test-lexer:
	cargo test lexer_tests

test-parser:
	cargo test parser_tests

test-binary:
	cargo test binary_tests

test-diff:
	cargo test diff_tests

test-integration:
	cargo test integration_tests

test-symbols:
	cargo test symbol_tests

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Run benchmarks
bench:
	cargo bench

# Run specific benchmarks
bench-lexer:
	cargo bench benchmark_lexer

bench-parser:
	cargo bench benchmark_parser

bench-diff:
	cargo bench benchmark_diff

# Clean build artifacts
clean:
	cargo clean

# Format code
fmt:
	cargo fmt

# Run linter
lint:
	cargo clippy -- -D warnings

# Type check without building
check:
	cargo check

# Run all checks (format, lint, type check, test)
ci: fmt lint check test

# Create example files and test CLI
examples: build-cli
	@echo "Creating example files..."
	@mkdir -p examples/generated
	@echo 'module Simple\nlet x = 42\nlet add = fun a b -> a + b' > examples/generated/simple.eff
	@echo 'module Modified\nlet x = 43\nlet add = fun a b -> a + b' > examples/generated/modified.eff
	@echo "Compiling examples..."
	./target/release/effect-cli compile -i examples/generated/simple.eff --timing
	./target/release/effect-cli compile -i examples/generated/modified.eff --timing
	@echo "Running diff..."
	./target/release/effect-cli diff examples/generated/simple.eff.bin examples/generated/modified.eff.bin --verbose
	@echo "Analyzing files..."
	./target/release/effect-cli analyze examples/generated/simple.eff.bin --hash --size

# Quick development cycle
dev: fmt check test-verbose

# Performance testing
perf: build
	@echo "Performance testing..."
	cargo test test_large_ast_performance -- --nocapture
	cargo test test_compression_ratio -- --nocapture
	cargo test test_diff_performance -- --nocapture

# Generate documentation
docs:
	cargo doc --open

# Install CLI globally
install: build
	cargo install --path . --bin effect-cli

# Show help
help:
	@echo "EffectLang Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  build         - Build the project in release mode"
	@echo "  build-cli     - Build only the CLI tool"
	@echo "  test          - Run all tests"
	@echo "  test-*        - Run specific test suites"
	@echo "  bench         - Run all benchmarks"
	@echo "  bench-*       - Run specific benchmarks"
	@echo "  clean         - Clean build artifacts"
	@echo "  fmt           - Format code"
	@echo "  lint          - Run linter"
	@echo "  check         - Type check without building"
	@echo "  ci            - Run all CI checks"
	@echo "  examples      - Create and test example files"
	@echo "  dev           - Quick development cycle"
	@echo "  perf          - Run performance tests"
	@echo "  docs          - Generate and open documentation"
	@echo "  install       - Install CLI globally"
	@echo "  help          - Show this help"

# Debug target for development
debug:
	RUST_LOG=debug cargo test test_simple_module_roundtrip -- --nocapture

# Memory usage analysis
memcheck: build
	valgrind --tool=massif ./target/release/effect-cli compile -i examples/simple.eff

# Code coverage (requires cargo-tarpaulin)
coverage:
	cargo tarpaulin --out Html

# Static analysis (requires cargo-audit)
audit:
	cargo audit

# Update dependencies
update:
	cargo update

# Check for outdated dependencies
outdated:
	cargo outdated