#!/bin/bash

DATA_FILE=mst.jsonl
DRAWING_FILE=mst.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE

	for n in 1000 2000 5000 10000 20000 50000 100000 200000 500000 1000000
	do
		echo "Benchmark MST with $n vertices"...
		for _ in {1..20}
		do
			./stt-benchmarks/target/release/bench_mst -n $n --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile mst-vertices --output-file results/$DRAWING_FILE
