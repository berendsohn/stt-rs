#!/bin/bash

echo "Building benchmarks..."
bash build_bench.sh

bash benchmark_queries_uniform.sh
bash benchmark_queries_uniform_large.sh
bash benchmark_queries_path_prob.sh
bash benchmark_degenerate.sh
bash benchmark_degenerate_noisy.sh
bash benchmark_mst.sh
bash benchmark_lca.sh
bash benchmark_lca_evert.sh

if [[ $1 == "--all" ]]
    bash benchmark_num_rotations.sh
    bash benchmark_fd_con.sh
then
