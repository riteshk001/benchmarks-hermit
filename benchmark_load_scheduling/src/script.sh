#!/bin/bash
> results.jsonl

for length in 64 128 256 512 1024; do
    for nb in 6 12 24 48 96; do
        for io in 0 600; do
            ./bench $nb $length $io fix >> results.jsonl
            ./bench $nb $length $io mix >> results.jsonl
        done
    done
done

python3 plot.py