import tsconj
from sys import argv

problem = argv[1]
max_solutions = int(argv[2])
model_path = argv[3]

spaces = tsconj.compute_trap_spaces(
	model_path, 
	max_output=max_solutions, 
	method="conj", 
	computation=problem
)

for space in spaces:
	print("".join(space))