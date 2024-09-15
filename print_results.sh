#!/bin/bash

DELIM="-------------------------------------"

echo "Uniformly random queries"
python3 show_benchmarks/tabulate.py --input-file results/queries_uniform.jsonl --key num_vertices --value-unit ns --value-per query --decimal-places 2

echo $DELIM
echo "Random queries with variable probability for compute_path_weight()"
python3 show_benchmarks/tabulate.py --input-file results/queries_path_prob.jsonl --key path_query_prob --value-unit us --value-per query --decimal-places 2

echo $DELIM
echo "Degenerate queries"
python3 show_benchmarks/tabulate.py --input-file results/degenerate.jsonl --key num_vertices --value-unit us --value-per query --decimal-places 2

echo $DELIM
echo "Noisy Degenerate queries"
python3 show_benchmarks/tabulate.py --input-file results/degenerate_noisy.jsonl --key std_dev --value-unit us --value-per query --decimal-places 2

echo $DELIM
echo "Minimum spanning forest"
python3 show_benchmarks/tabulate.py --input-file results/mst.jsonl --key num_vertices --value-unit us --value-per edge --decimal-places 2

echo $DELIM
echo "Minimum spanning forest (ogbl-collab dataset)"
python3 show_benchmarks/tabulate.py --input-file results/mst_ogbl.jsonl --key num_vertices --value-unit us --value-per edge --decimal-places 2

echo $DELIM
echo "LCA"
python3 show_benchmarks/tabulate.py --input-file results/lca.jsonl --key num_vertices --value-unit us --value-per query

echo $DELIM
echo "LCA/Evert"
python3 show_benchmarks/tabulate.py --input-file results/lca_evert.jsonl --key num_vertices --value-unit us --value-per query


echo $DELIM
echo "Rotation count"
python3 show_benchmarks/tabulate.py --input-file results/num_rotations.jsonl --key num_vertices --value rotation_count --value-unit rots --value-per query --decimal-places 3
