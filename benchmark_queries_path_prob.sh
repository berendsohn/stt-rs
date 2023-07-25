#!/bin/bash

DATA_FILE=queries_path_prob.jsonl
DRAWING_FILE=queries_path_prob.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE
	
	n=5000
	q=$((100*n))

	for p in 0 0.2 0.4 0.6 0.8 0.9 0.95 1
	do
		echo "Benchmark queries with $n vertices, $q queries, and path-query probability $p..."
		for _ in {1..5}
		do
			s=$RANDOM
			echo "  seed=$s"
			./stt-benchmarks/target/release/bench_queries -s $s -n $n -q $q -p $p --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE && exit
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile queries-path-prob --output-file results/$DRAWING_FILE
