#!/bin/bash

trap 'echo "Received signal, exiting..."; exit 0' SIGINT SIGTERM

set -e
set -x

mkdir -p ./results

TOOL='sybila/tool-tsconj'
BENCHMARKS='./test_instances/_normalized'
TIMEOUT=${TIMEOUT:-'10s'}
PARALLEL=${PARALLEL:-'1'}

# Time to one solution across various semantics.
python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.bnet' --parallel $PARALLEL -- --count-only /sw/run.py fix 0
for d in run_*/; do mv -- "$d" "results/tsconj_2v_${d#./}"; done

python3 ./benchmarks/bench_docker.py --docker-image $TOOL --timeout $TIMEOUT --folder $BENCHMARKS --match '.*.bnet' --parallel $PARALLEL -- --count-only /sw/run.py min 0
for d in run_*/; do mv -- "$d" "results/tsconj_prf_${d#./}"; done
