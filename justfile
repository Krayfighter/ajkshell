
clean: clean_build_dir
    cargo clean

clean_build_dir:
    rm -rf build
    mkdir build

build_rust_debug:
    cargo build --all
    cp -R target/debug/* build

build_go_echo:
    go build -C src/echo -o ../../build

debug: clean_build_dir build_rust_debug build_go_echo
    # rm -rf build
    # mkdir build
    # cargo build --all
    # cp -R target/debug/* build
    build/main



