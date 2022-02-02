default:
    @just --list

test:
    cargo test --verbose --workspace --exclude py-core

build_web_core:
    cd web-core && wasm-pack build -s deep-trinity -d ../web/packages/web-core

build_python:
    cd "{{ justfile_directory() }}/py-core" && maturin build --no-sdist

develop_python:
    cd "{{ justfile_directory() }}/py-core" && maturin develop

prepare_python_tests:
    cd "{{ justfile_directory() }}/py-core/python-tests" && poetry run bash -lc 'just -f {{ justfile() }} develop_python'

test_python:
    cd "{{ justfile_directory() }}/py-core/python-tests" && poetry run pytest
