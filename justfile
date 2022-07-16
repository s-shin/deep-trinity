crates_dir := justfile_directory() + '/crates'
web_dir := justfile_directory() + '/web'
tools_dir := justfile_directory() + '/tools'

default:
    @just --list

test:
    cargo test --verbose --workspace --exclude py-core
    cargo test --verbose -p py-core --no-default-features
    just prepare_and_test_python_core

build_web_core:
    cd '{{ crates_dir }}/web-core' && wasm-pack build -s deep-trinity -d '{{ web_dir }}/packages/web-core'

build_python_core:
    cd '{{ crates_dir }}/py-core' && maturin build --no-sdist

install_python_core:
    cd '{{ crates_dir }}/py-core' && maturin develop

prepare_python_core_tests:
    cd '{{ crates_dir }}/py-core/python-tests' \
    && poetry install \
    && . "$(poetry env info --path)/bin/activate" \
    && just -f '{{ justfile() }}' install_python_core

test_python_core:
    cd '{{ crates_dir }}/py-core/python-tests' && poetry run pytest

prepare_and_test_python_core: prepare_python_core_tests test_python_core

prepare_ppt_capture:
    cd '{{ tools_dir }}/ppt-capture' \
    && . "$(poetry env info --path)/bin/activate" \
    && just -f '{{ justfile() }}' install_python_core
