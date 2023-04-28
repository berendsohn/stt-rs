#!/bin/bash

python3 -c "import matplotlib" &>/dev/null || echo "WARNING: python3 or matplotlib not found. Figures will not be generated."

echo "Building benchmarks..."
bash build_bench.sh

bash benchmark_queries_uniform.sh
bash benchmark_queries_uniform_large.sh
bash benchmark_degenerate.sh
bash benchmark_degenerate_noisy.sh
bash benchmark_mst.sh
