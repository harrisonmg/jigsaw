#!/bin/bash
cd "${0%/*}"
while true; do
  ./server || sleep 10
done
