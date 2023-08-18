#!/bin/bash
cd "${0%/*}"
trunk build
cargo run --bin server $@
