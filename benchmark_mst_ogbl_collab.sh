#!/bin/bash

. util.sh

INPUT_FILE=data/ogbl-collab/incremental_mst.txt
DATA_FILE=mst_ogbl.jsonl
REPEAT=5


if [ -f $INPUT_FILE ]
then
    mkdir -p results
    rm -f results/$DATA_FILE
    
    echo "Benchmark MST on ogbl-collab dataset..."
    progress_bar_start
    for _ in $(eval echo {1..$REPEAT})
    do
        s=$RANDOM
        ./stt-benchmarks/target/release/bench_mst -i $INPUT_FILE --json >> results/$DATA_FILE || exit
        progress_bar_tick
    done
    progress_bar_end
else
    echo "Dataset file $INPUT_FILE does not exist"
fi
