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
		for _ in {1..20}
		do
			s=$RANDOM
			echo "  seed=$s"
			./stt-benchmarks/target/release/bench_num_rotations --json --seed $s -n $n -q $q >> results/$DATA_FILE
		done
	done
fi

#python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile queries-path-prob --output-file results/$DRAWING_FILE
