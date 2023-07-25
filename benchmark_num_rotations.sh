#!/bin/bash

DATA_FILE=num_rotations.jsonl
DRAWING_FILE=num_rotations.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE
	
	for n in 1000 1500 2000
	do
		q=$((n*n))
		echo "Count rotations with $q queries on $n vertices..."
		for _ in {1..5}
		do
			s=$RANDOM
			echo "  seed=$s"
			./stt-benchmarks/target/release/bench_num_rotations --json -s $s -n $n -q $q >> results/$DATA_FILE || exit
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile num_rotations --output-file results/$DRAWING_FILE
