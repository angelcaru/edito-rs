#!/bin/sh

if [ $# -ne 2 ]; then
    echo "Usage: $0 <src> <out>"
    exit 1
fi

SRC=$1
OUT=$2
gcc -ggdb -fPIC -shared -o lib$OUT.so $SRC

