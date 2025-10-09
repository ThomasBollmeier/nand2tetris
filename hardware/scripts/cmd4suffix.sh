#!/bin/bash

cmd=$1

files=`ls *.$2`

for file in $files; do
    [ -e "$file" ] || continue
    # Replace the following echo command with your desired command
    echo "Running: $cmd $file"
    "$cmd" "$file"
done
