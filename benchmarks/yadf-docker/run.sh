#!/bin/sh

set -e
set -x

java -jar /sw/yadf_0.1.1.jar $1 $2 > /tmp/problem.lp 
/sw/lpopt-2.2-x86_64/lpopt < /tmp/problem.lp > /tmp/translated.lp 
clingo -n $3 /tmp/translated.lp