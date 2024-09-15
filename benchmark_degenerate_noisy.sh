#!/bin/bash

. util.sh

DATA_FILE=degenerate_noisy.jsonl
REPEAT=10

mkdir -p results
rm -f results/$DATA_FILE

n=5000

for d in 0 1 2 5 10 20 50 100 150 200 250 300
do
    echo "Benchmark degenerate queries with $n vertices and std-dev $d..."
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        if (($d == 0))
        then
            ./stt-benchmarks/target/release/bench_degenerate -n $n -d $d --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        else
            s=$RANDOM
            ./stt-benchmarks/target/release/bench_degenerate -s $s -n $n -d $d --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        fi
        progress_bar_tick
    done
    progress_bar_end
done
