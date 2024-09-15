#!/bin/bash

. util.sh

DATA_FILE=degenerate.jsonl
REPEAT=10

mkdir -p results
rm -f results/$DATA_FILE

for n in 100 200 500 1000 2000 5000 10000
do
    echo "Benchmark degenerate queries with $n vertices"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        ./stt-benchmarks/target/release/bench_degenerate -n $n --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
