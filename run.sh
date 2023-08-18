#!/bin/bash
cd "${0%/*}"
trunk build --release
cargo run --release --bin server $@
