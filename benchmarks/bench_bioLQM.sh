#!/bin/bash

trap 'echo "Received signal, exiting..."; exit 0' SIGINT SIGTERM

set -e
set -x

mkdir -p ./results

TOOL='sybila/tool-biolqm'
BENCHMARKS='./test_instances/_normalized'
TIMEOUT=${TIMEOUT:-'10s'}
PARALLEL=${PARALLEL:-'1'}

# Time to one solution across various semantics.
python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.sbml' --parallel $PARALLEL -- -r stable BDD
for d in run_*/; do mv -- "$d" "results/biolqm_2v_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.sbml' --parallel $PARALLEL -- -r trapspace terminal BDD
for d in run_*/; do mv -- "$d" "results/biolqm_prf_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.sbml' --parallel $PARALLEL -- -r trapspace tree BDD
for d in run_*/; do mv -- "$d" "results/biolqm_com_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.sbml' --parallel $PARALLEL -- -r trapspace all BDD
for d in run_*/; do mv -- "$d" "results/biolqm_adm_${d#./}"; done

