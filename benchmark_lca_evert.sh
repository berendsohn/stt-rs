#!/bin/bash

. util.sh

DATA_FILE=lca_evert.jsonl
REPEAT=10

MAX_FOR_SIMPLE=50000

mkdir -p results
rm -f results/$DATA_FILE

for n in 10000 20000 50000 100000 500000
do
    let q=$n*10
    echo -n "Benchmark LCA/Evert with $n vertices and $q queries"
    if (( n > MAX_FOR_SIMPLE ))
    then
        echo -n " (excluding simple implementation)"
    fi
    echo "..."
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        if (( n <= MAX_FOR_SIMPLE ))
        then
            ./stt-benchmarks/target/release/bench_lca -e -s $s -n $n -q $q --json link-cut stable-greedy-splay stable-two-pass-splay stable-local-two-pass-splay stable-move-to-root simple >> results/$DATA_FILE || exit
        else # Exclude simple impl
            ./stt-benchmarks/target/release/bench_lca -e -s $s -n $n -q $q --json link-cut stable-greedy-splay stable-two-pass-splay stable-local-two-pass-splay stable-move-to-root >> results/$DATA_FILE || exit
        fi
        progress_bar_tick
    done
    progress_bar_end
done
