#!/bin/bash
#:
#: name = "build-and-test"
#: variety = "basic"
#: target = "helios-2.0"
#: rust_toolchain = "stable"
#: output_rules = [
#:   "/work/debug/*",
#:   "/work/release/*",
#: ]
#:

set -o errexit
set -o pipefail
set -o xtrace

cargo --version
rustc --version
cargo install cargo-nextest

echo "##### check #####"
cargo fmt -- --check
cargo check
cargo clippy --all-targets -- --deny warnings

echo "##### build #####"
cargo build
cargo build --release

echo "##### test #####"
cargo nextest run

if [[ -z $CI ]]; then
    exit 0;
fi

for x in debug release
do
    mkdir -p /work/$x
    cp target/$x/isf /work/$x/
done

