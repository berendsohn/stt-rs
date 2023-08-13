#!/bin/bash

. util.sh

DATA_FILE=queries_uniform_large.jsonl
REPEAT=20

mkdir -p results
rm -f results/$DATA_FILE

for n in 2000 3000 4000 5000 6000 7000 8000
do
    q=$((100*n))
    echo "Benchmark $q queries with $n vertices"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
