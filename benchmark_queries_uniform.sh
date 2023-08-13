#!/bin/bash

. util.sh

DATA_FILE=queries_uniform.jsonl
REPEAT=20

mkdir -p results
rm -f results/$DATA_FILE

for n in 500 1000
do
    q=$((20*n))
    echo "Benchmark queries with $n vertices"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut petgraph-dynamic >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
