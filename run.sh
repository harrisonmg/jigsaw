#!/bin/bash
cd "${0%/*}"

./server "$@"

while true; do
  ./server puzzle_backup.json
done
