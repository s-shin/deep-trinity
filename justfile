default:
    @just --list

test:
    cargo test --verbose --workspace --exclude py-core
    just prepare_python_tests test_python

build_web_core:
    cd web-core && wasm-pack build -s deep-trinity -d ../web/packages/web-core

build_python:
    cd '{{ justfile_directory() }}/py-core' && maturin build --no-sdist

develop_python:
    cd '{{ justfile_directory() }}/py-core' && maturin develop

prepare_python_tests:
    cd '{{ justfile_directory() }}/py-core/python-tests' \
    && poetry install \
    && source "$(poetry env info --path)/bin/activate" \
    && just -f '{{ justfile() }}' develop_python

test_python:
    cd '{{ justfile_directory() }}/py-core/python-tests' && poetry run pytest

prepare_and_test_python: prepare_python_tests test_python
