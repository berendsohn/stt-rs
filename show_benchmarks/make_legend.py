import visualize

import matplotlib.pyplot as plt

def main() :
	for impl, color in visualize.ALGORITHM_COLORS.items() :
		if impl != "Kruskal (petgraph)" :
			if impl == "Petgraph" :
				impl = "Petgraph or Kruskal"
			plt.errorbar( [0], [0], yerr = [[0], [0]], capsize = 2, label = impl, color = color )

	plt.legend()
	plt.show()

if __name__ == "__main__" :
	main()
