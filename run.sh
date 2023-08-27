#!/bin/bash
cd "${0%/*}"
while true; do
  ./server queue.txt || sleep 10
done
