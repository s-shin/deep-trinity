.PHONY: all
all:

.PHONY: build_web_core
build_web_core:
	cd web-core && wasm-pack build -s deep-trinity -d ../web/packages/web-core
