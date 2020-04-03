.PHONY: all
all:

.PHONY: use_nightly_rust
use_nightly_rust:
	rustup override add nightly

.PHONY: use_stable_rust
use_stable_rust:
	rustup override unset

.PHONY: build_web_core
build_web_core:
	cd web-core && wasm-pack build -s deep-trinity -d ../web/packages/web-core

.PHONY: run_python
run_python:
	mkdir -p tmp/py
	rm -f tmp/py/detris.so
	ln -s ../../target/debug/deps/libdetris.dylib tmp/py/detris.so
	PYTHONPATH=tmp/py python $(ARGS)
