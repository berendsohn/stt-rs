#!/bin/bash

DATA_FILE=degenerate_noisy.jsonl
DRAWING_FILE=degenerate_noisy.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE

	n=5000

	for d in 0 1 2 5 10 20 50 100 150 200 250 300
	do
		echo "Benchmark degenerate queries with $n vertices and std-dev $d..."
		for _ in {1..5}
		do
			if (($d == 0)) then
				./stt-benchmarks/target/release/bench_degenerate -n $n -d $d --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
			else
				s=$RANDOM
				echo "  seed=$s"
				./stt-benchmarks/target/release/bench_degenerate -s $s -n $n -d $d --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
			fi
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile degenerate-noisy --output-file results/$DRAWING_FILE
