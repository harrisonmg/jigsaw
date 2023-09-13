#!/bin/bash
cd "${0%/*}"

./server "$@"

while true; do
  ./server
done
