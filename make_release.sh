#!/bin/bash
cd "${0%/*}"
trunk build --release
cargo build --release --bin server
rm -f jigsaw.zip
zip jigsaw.zip -j target/release/server multi.sh
zip jigsaw.zip -r dist/
