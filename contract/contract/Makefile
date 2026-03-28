CARGO_TARGET_DIR := $(abspath $(CURDIR)/../target)

default: build

all: test

test: build
	cargo test

build:
	CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) stellar contract build
	@ls -l $(CARGO_TARGET_DIR)/wasm32v1-none/release/*.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean
