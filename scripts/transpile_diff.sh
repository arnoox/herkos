#!/usr/bin/env bash
exec python3 "$(dirname "$0")/transpile_diff.py" "$@"
