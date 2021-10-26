import matplotlib.pyplot as plt
import numpy as np
#from io import StringIO

txtdata = open("pop10_gen500.txt")
data = np.loadtxt(txtdata, usecols=None, max_rows=25)

plt.plot(data, '.')
plt.xlabel('Generation Number')
plt.ylabel('Individual Fitness Score')
plt.title("Traveling Salesman Genetic Algorithm\nPopulation Size:10, First 25 Generations")
plt.show()