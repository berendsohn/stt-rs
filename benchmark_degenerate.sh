#!/bin/bash

DATA_FILE=degenerate.jsonl
DRAWING_FILE=degenerate.pdf

if [ "$1" != "--only-plot" ]; then
	SEED=0

	mkdir -p results
	rm -f results/$DATA_FILE

	for n in 10 20 50 100 200 500 1000 2000 5000 10000
	do
		echo "Benchmark degenerate queries with $n vertices"...
		for _ in {1..5}
		do
			./stt-benchmarks/target/release/bench_degenerate -s $SEED -n $n --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile degenerate --output-file results/$DRAWING_FILE
