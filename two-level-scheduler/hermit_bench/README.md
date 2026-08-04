# Quick Guide - Benchmark

This program measures the performance of the scheduler in HermitOS. It creates many tasks that run in parallel and measures:

Waiting time: delay between creation and start of execution

Execution time: actual work duration

Response time: total time from creation to completion

The benchmark tests different configurations:

Task counts: 50, 100, 250, 300, 400

Work lengths: 64 to 2048 (computation units)

5 runs per configuration for reliability

Each task is a thread that simulates CPU work (matrix multiplication) with a small I/O pause.

## Terminal Output
The program outputs one JSON object per benchmark run. Each line is valid JSON:

json
{"scenario":"mix","nb_tasks":50,"length":0,"length_short":32,"length_long":64,"io_time_ms":600,"io_time_short_ms":0,"cores":16,"total_time_ms":42,"waiting_time":{"min":0,"max":1,"mean":0},"execution_time":{"min":16,"max":39,"mean":23},"response_time":{"min":17,"max":40,"mean":24},"run":1}
{"scenario":"mix","nb_tasks":50,"length":0,"length_short":48,"length_long":96,"io_time_ms":600,"io_time_short_ms":0,"cores":16,"total_time_ms":49,"waiting_time":{"min":0,"max":1,"mean":0},"execution_time":{"min":18,"max":49,"mean":32},"response_time":{"min":19,"max":50,"mean":33},"run":1}
...