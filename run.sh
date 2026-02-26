#!/bin/bash
cd "${0%/*}"

./server "$@"

while true; do
  if [ -f puzzle_backup.json ]; then
    ./server puzzle_backup.json
  else
    ./server
  fi
done
