#!/bin/bash

. util.sh

DATA_FILE=mst.jsonl
REPEAT=10

mkdir -p results
rm -f results/$DATA_FILE

for n in 1000 2000 5000 10000 20000 50000 100000 200000 500000 1000000
do
    echo "Benchmark MST with $n vertices"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_mst -s $s -n $n --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
