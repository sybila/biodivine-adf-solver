#!/bin/bash

set -e
set -x

mkdir -p ./results

# Time to one solution across various semantics.
python3 ./benchmarks/run.py 1200s ./test_instances ./benchmarks/run_yadf_adm_1.sh
mv _run_* ./results/yadf_1_admissible
python3 ./benchmarks/run.py 1200s ./test_instances ./benchmarks/run_yadf_com_1.sh
mv _run_* ./results/yadf_1_complete
python3 ./benchmarks/run.py 1200s ./test_instances ./benchmarks/run_yadf_prf_1.sh
mv _run_* ./results/yadf_1_preferred
python3 ./benchmarks/run.py 1200s ./test_instances ./benchmarks/run_yadf_stb_1.sh
mv _run_* ./results/yadf_1_stable