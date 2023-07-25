#!/bin/bash

DATA_FILE=lca.jsonl
DRAWING_FILE=lca.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE

	for n in 1000 2000 5000 10000 20000 50000 100000 200000 500000 1000000
	do
		let q=$n*10
		echo "Benchmark LCA with $n vertices and $q queries"...
		for _ in {1..5}
		do
			s=$RANDOM
			echo "  seed=$s"
			./stt-benchmarks/target/release/bench_lca -s $s -n $n -q $q --json >> results/$DATA_FILE || exit
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile lca --output-file results/$DRAWING_FILE --exclude Simple
