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
	./script/install_py_core.sh debug tmp/py
	PYTHONPATH=tmp/py python $(ARGS)
