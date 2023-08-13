#!/bin/bash

if ! python3 -c "import matplotlib" &>/dev/null
then
    echo "WARNING: python3 or matplotlib not found. Cannot generate figures."
    exit
fi

python3 show_benchmarks/visualize.py --input-file results/queries_uniform.jsonl --profile queries-uniform
python3 show_benchmarks/visualize.py --input-file results/queries_uniform_large.jsonl --profile queries-uniform
python3 show_benchmarks/visualize.py --input-file results/queries_path_prob.jsonl --profile queries-path-prob

python3 show_benchmarks/visualize.py --input-file results/degenerate.jsonl --profile degenerate
python3 show_benchmarks/visualize.py --input-file results/degenerate_noisy.jsonl --profile degenerate-noisy

python3 show_benchmarks/visualize.py --input-file results/mst.jsonl --profile mst-vertices

python3 show_benchmarks/visualize.py --input-file results/lca.jsonl --profile lca
python3 show_benchmarks/visualize.py --input-file results/lca_evert.jsonl --profile lca_evert

python3 show_benchmarks/visualize.py --input-file results/fd_con.jsonl --profile fd-con
python3 show_benchmarks/visualize.py --input-file results/num_rotations.jsonl --profile num_rotations
