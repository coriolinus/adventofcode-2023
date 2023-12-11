#!/usr/bin/env bash

set -eu

cd "$(dirname "$0")"
cargo build --release

for day_n in $(toml2json Cargo.toml | jq -r '.workspace.members[]'); do
    hyperfine \
        --warmup 10 \
        --min-runs 20 \
        --shell none \
        --style color \
        --export-markdown "$day_n/bench.md" \
        --command-name "$day_n part 1" \
        "target/release/$day_n" \
        --command-name "$day_n part 2" \
        "target/release/$day_n --part2 --no-part1"

done
