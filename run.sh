#!/bin/bash
cd "${0%/*}"
trunk build
PORT=8080 cargo run --bin server $@
