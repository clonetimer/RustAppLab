#!/bin/sh
set -eu
cd "$(dirname "$0")"
if [ "$#" -eq 0 ]; then set -- --open; fi
if [ -x ./bin/labquest ]; then
    exec ./bin/labquest workbench "$@"
fi
exec cargo run --locked --offline -p labquest -- workbench "$@"
