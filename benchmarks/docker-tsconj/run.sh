#!/bin/bash

# All arguments are passed on to Python interpreter with tsconj installed

trap 'echo "Received signal, exiting..."; exit 0' SIGINT SIGTERM

set -x
set -e

if [ "$1" = "--count-only" ]; then
    /sw/venv/bin/python3 "${@:2}" | wc -l
else
    /sw/venv/bin/python3 "$@"
fi