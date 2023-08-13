#!/bin/bash

. util.sh

DATA_FILE=fd_con.jsonl
REPEAT=20

mkdir -p results
rm -f results/$DATA_FILE

for n in 500 750 1000 1250 1500
do
    let q=$n*$n/2
    echo "Benchmark fully-dynamic connectivity on $n vertices with $q queries"...
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_fd_con -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
