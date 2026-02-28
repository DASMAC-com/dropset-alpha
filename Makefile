.PHONY: build build-deps build-sbf check clean test clippy format help 

# Full builds
build-deps:
	cargo build --workspace

# Solana BPF build
build-sbf:
	cd program && cargo build-sbf

build: build-deps build-sbf

# Development commands
check: 
	cargo check

clean:
	cargo clean

test:
	cargo test

# Help
help:
	@echo "Available targets:"
	@echo "  build-deps         - Build all dependencies in workspace"
	@echo "  build              - Full build, including Solana program"
	@echo "  build-sbf          - Build Solana BPF program"
	@echo "  check              - cargo check"
	@echo "  clean              - cargo clean"
	@echo "  test               - cargo test"
