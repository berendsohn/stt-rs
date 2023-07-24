#!/bin/bash

DATA_FILE=lca.jsonl
DRAWING_FILE=lca.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE

	for n in 1000 2000 5000 10000 20000 50000 100000 #200000 500000 1000000
	do
		let q=$n*10
		echo "Benchmark LCA with $n vertices and $q queries"...
		for _ in {1..5}
		do
			./stt-benchmarks/target/release/bench_rooted -n $n -q $q --print #--json >> results/$DATA_FILE
		done
	done
fi

#python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile mst-vertices --output-file results/$DRAWING_FILE
