#!/bin/bash

. util.sh

DATA_FILE=queries_path_prob.jsonl
REPEAT=20

mkdir -p results
rm -f results/$DATA_FILE

n=5000
q=$((100*n))

for p in 0 0.2 0.4 0.6 0.8 0.9 0.95 1
do
    echo "Benchmark queries with $n vertices, $q queries, and path-query probability $p..."
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q -p $p --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
