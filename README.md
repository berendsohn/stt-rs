# STT dynamic forest library and benchmarks

The library source code is contained in the `stt` directory. See `stt/README.md` for instructions on how to build the library and its documentation.

## Benchmarks

All benchmarks for the paper can be executed by running
```
./all_benchmarks.sh
```

This will take a while. Results are written to the `results` directory.

The benchmarks can be run individually using the `benchmark_*.sh` scripts. There are two additional benchmarks not used in the paper and not automatically executed by `all_benchmarks.sh`, namely `benchmark_num_rotations.sh` which compares the number of rotations the various STT-based implementations need, and `benchmark_fd_con.sh`, which 

After running benchmarks, all benchmarks can be plotted in interactive windows (vai `mathplotlib`) using
```
./show_results.sh
```
This will skip all benchmarks that haven't been executed before. To just show a list of results as text, use
```
./print_results.sh
```

For more detailed options, after building the benchmarks using `./build_bench.sh`, executables can be called directly from the `stt-benchmarks/target/release` directory. Command-line help is available (including some options not used in the paper). Generation of plots can also be manually adjusted by running
```
python3 show_benchmarks/visualize.py [...]
```

Again, command-line help is available.
