#!/bin/bash

. util.sh

DATA_FILE=num_rotations.jsonl
REPEAT=20

mkdir -p results
rm -f results/$DATA_FILE

for n in 1000 2000 3000 4000 5000
do
    q=$((100*n))
    echo "Count rotations with $q queries on $n vertices..."
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_num_rotations --json -s $s -n $n -q $q >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
done
