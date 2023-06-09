#!/bin/bash

python3 show_benchmarks/visualize.py --input-file results/queries_uniform.jsonl --profile queries-uniform
python3 show_benchmarks/visualize.py --input-file results/queries_uniform_large.jsonl --profile queries-uniform

python3 show_benchmarks/visualize.py --input-file results/degenerate.jsonl --profile degenerate
python3 show_benchmarks/visualize.py --input-file results/degenerate_noisy.jsonl --profile degenerate-noisy

python3 show_benchmarks/visualize.py --input-file results/mst.jsonl --profile mst-vertices
