import pandas as pd
import matplotlib.pyplot as plt

# Read CSV (replace with your filename)
data_fork = pd.read_csv("/home/karki/hermit/benchmarks_local/benchmark.csv")
data_spawn = pd.read_csv("/home/karki/hermit/benchmarks_local/benchmark_spawn.csv")

# Plot
avg_fork_time = data_fork["avg_fork_time"]
#print(f"avg_fork_time{avg_fork_time}")
plt.figure(figsize=(10, 6))
plt.plot(data_fork["memory_size_mb"], data_fork["avg_fork_time"],
         marker='o', linestyle='-', color='b', label="Average Latency")
plt.plot(data_fork["memory_size_mb"], data_fork["min_fork_time"],
         marker='x', linestyle='--', color='g', label="Min Latency")
plt.plot(data_fork["memory_size_mb"], data_fork["max_fork_time"],
         marker='x', linestyle='--', color='r', label="Max Latency")

# Customize plot
plt.xlabel("Parent Process Memory Size (MB)")
plt.ylabel("Fork() Latency (us)")
plt.title("Fork() Latency vs. Parent Process Memory Size")
plt.grid(True, linestyle='--', alpha=0.7)
plt.legend()
#plt.tight_layout()

# Save and show
plt.savefig("fork_latency_vs_memory.png", dpi=300)
plt.show()

plt.figure(figsize=(10, 6))
plt.plot(data_spawn["memory_size_mb"], data_spawn["avg_spawn_time"],
         marker='o', linestyle='-', color='b', label="Average Latency")
plt.plot(data_spawn["memory_size_mb"], data_spawn["min_spawn_time"],
         marker='x', linestyle='--', color='g', label="Min Latency")
plt.plot(data_spawn["memory_size_mb"], data_spawn["max_spawn_time"],
         marker='x', linestyle='--', color='r', label="Max Latency")

# Customize plot
plt.xlabel("Parent Process Memory Size (MB)")
plt.ylabel("spawn() Latency (ns)")
plt.title("spawn() Latency vs. Parent Process Memory Size")
plt.grid(True, linestyle='--', alpha=0.7)
plt.legend()
#plt.tight_layout()


plt.savefig("spawn_latency_vs_memory.png", dpi=300)
