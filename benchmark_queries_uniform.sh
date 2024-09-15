#!/bin/bash

. util.sh

DATA_FILE=queries_uniform.jsonl
REPEAT=10

mkdir -p results
rm -f results/$DATA_FILE

for n in 500 1000 2000 5000 10000 50000 100000 500000
do
    q=$((10*n))
    echo "Benchmark queries with $n vertices"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        if (( n <= 1000 )); then
            ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut petgraph-dynamic >> results/$DATA_FILE || exit
        elif (( n <= 50000 )); then
            ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        else
            ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root >> results/$DATA_FILE || exit
        fi
        progress_bar_tick
    done
    progress_bar_end
done
