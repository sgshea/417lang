#!/bin/bash

# Default command
command=""

if [[ "$#" -gt 0 ]]; then
  case "$1" in
    "-p")
      command="cargo run -p parser --release"
      ;;
    "-a")
      command="cargo run -p interpreter --release --features "parser""
      ;;
    "-t")
      command="cargo test --release"
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
else
  # Defualt command
  command="cargo run -p interpreter --release"
fi

# Execute the command
$command