#!/bin/bash

trap 'echo "Received signal, exiting..."; exit 0' SIGINT SIGTERM

set -e
set -x

mkdir -p ./results

TOOL='sybila/tool-aeon-py'
BENCHMARKS='./test_instances/_normalized'
TIMEOUT=${TIMEOUT:-'10s'}
PARALLEL=${PARALLEL:-'1'}

# Time to one solution across various semantics.
python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.bnet' --parallel $PARALLEL -- /sw/adf_two_valued.py
for d in run_*/; do mv -- "$d" "results/aeon_2v_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.bnet' --parallel $PARALLEL -- /sw/adf_complete.py
for d in run_*/; do mv -- "$d" "results/aeon_com_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.bnet' --parallel $PARALLEL -- /sw/adf_preferred.py
for d in run_*/; do mv -- "$d" "results/aeon_prf_${d#./}"; done
