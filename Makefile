.PHONY: all
all:

.PHONY: build_core_wasm
build_core_wasm:
	cd core-wasm && wasm-pack build -s deep-trinity -d ../web/packages/core-wasm
