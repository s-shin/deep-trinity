.PHONY: all
all:

.PHONY: test
test:
	cargo test --verbose --workspace --exclude py-core

.PHONY: build_web_core
build_web_core:
	cd web-core && wasm-pack build -s deep-trinity -d ../web/packages/web-core

.PHONY: run_python
run_python:
	mkdir -p tmp/py
	rm -f tmp/py/detris.so
	ln -s ../../target/debug/deps/libdetris.dylib tmp/py/detris.so
	PYTHONPATH=tmp/py python $(ARGS)
