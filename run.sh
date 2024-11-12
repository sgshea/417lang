#!/bin/bash

# Default command
command=""

if [[ "$#" -gt 0 ]]; then
  case "$1" in
    "-p")
      command="cargo run -p parser --release"
      ;;
    "-a")
      command="cargo run -p interpreter --release -F "parser""
      ;;
    "-t")
      command="cargo test --release -F "parser""
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
else
  # Defualt command
  command="cargo run -q -p interpreter --release"
fi

# Shift arguments so that we can pass additional args
shift

# Execute the command
eval "$command" "$@"