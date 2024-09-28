


debug:
    rm -rf build
    mkdir build
    cargo build --all
    cp -R target/debug/* build
    build/main



