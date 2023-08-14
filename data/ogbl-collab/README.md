## Generating the ogbl-collab input

First, download the data set as described [here](https://ogb.stanford.edu/docs/linkprop/#ogbl-collab);

From the downloaded data, extract the files `edge.csv`, `edge_weight.csv`, and `edge_year.csv`. Then, run `./process.py`. This produces the file `incremental_mst.txt` that is used by the benchmark.
