#!/bin/bash
cd "${0%/*}"
while read entry; do
  entry=( $entry )
  piece_count="${entry[0]}"
  image="${entry[1]}"
  ./server "$piece_count" "$image"
done < <(tail -f images.txt)
