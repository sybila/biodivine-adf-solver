#!/bin/bash

# All arguments are passed on to goDiamond
# If first argument is --count-only, output is piped to wc -l

trap 'echo "Received signal, exiting..."; exit 0' SIGINT SIGTERM

set -x
set -e

/sw/venv/bin/python3 "$@"