#!/bin/bash
set -ex
cd "${0%/*}"
trunk build --release
cargo build --release -p server --target x86_64-unknown-linux-musl
rm -f jigsaw.zip
zip jigsaw.zip -j target/x86_64-unknown-linux-musl/release/server run.sh
zip jigsaw.zip -r dist/
