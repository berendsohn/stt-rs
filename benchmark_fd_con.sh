#!/bin/bash

DATA_FILE=fd_con.jsonl
DRAWING_FILE=fd_con.pdf

if [ "$1" != "--only-plot" ]; then
	mkdir -p results
	rm -f results/$DATA_FILE

	for n in 500 750 1000 1250 1500
	do
		let q=$n*$n
		echo "Benchmark fully-dynamic connectivity on $n vertices with $q queries"...
		for _ in {1..5}
		do
			s=$RANDOM
			echo "  seed=$s"
			./stt-benchmarks/target/release/bench_fd_con -s $s -n $n -q $q --json link-cut greedy-splay stable-greedy-splay two-pass-splay stable-two-pass-splay local-two-pass-splay local-stable-two-pass-splay move-to-root stable-move-to-root one-cut >> results/$DATA_FILE || exit
		done
	done
fi

python3 show_benchmarks/visualize.py --input-file results/$DATA_FILE --profile fd-con --output-file results/$DRAWING_FILE
