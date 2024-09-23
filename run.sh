#!/bin/bash

# Default command
command="cargo run -p interpreter --release"

# Check if "-p" flag is present
if [[ "$#" -gt 0 && "$1" == "-p" ]]; then
  command="cargo run -p interpreter --release --features "parser""
fi

# Execute the command
$command