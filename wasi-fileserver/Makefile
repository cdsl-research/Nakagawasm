BIN := target/wasm32-wasi/release/wasi-fileserver.wasm

$(BIN): src/lib.rs Cargo.toml Cargo.lock
	cargo wasi build --release

check:
	cargo wasi check --release

build:
	cargo wasi build --release

run: $(BIN)
	wasmtime run $(BIN)

.PHONY: run build
