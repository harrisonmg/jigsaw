#!/bin/bash
set -ex

cd "${0%/*}"

if  [[ -n "$@" ]]; then
  ./server "$@"
elif [[ -f puzzle_backup.json ]]; then
  ./server puzzle_backup.json
else
  ./server
fi
