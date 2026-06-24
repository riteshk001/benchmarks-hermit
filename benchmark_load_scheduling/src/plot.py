
import json
import os
import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict

# ============================================================
# CONFIGURATION
# ============================================================
INPUT_FILE = "results.jsonl"
NB_TASKS_VALUES = [50, 100, 250, 400]
LENGTH_VALUES = [128, 256, 512, 1024, 2048]
RESULTS_DIR = "results"

IO_CONFIG = {
    0:   {"linestyle": "-",  "color": "#1f77b4", "label": "IO=0ms",  "marker": "o"},
    850: {"linestyle": "--", "color": "#ff7f0e", "label": "IO=850ms", "marker": "s"},
}


SCENARIO_COLORS = {
    "fix": "#2ca02c",  
    "mix": "#d62728",  
}


def load_data(filepath):

    data = []
    with open(filepath, "r") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            data.append(json.loads(line))
    return data


def aggregate_runs(data):
    """Agrège les runs par (scenario, length, nb_tasks, io_time)."""
    grouped = defaultdict(list)
    for d in data:
        key = (
            d["scenario"],
            d.get("length") or d.get("length_long"),
            d["nb_tasks"],
            d["io_time_ms"],
        )
        grouped[key].append(d)
    
    agg = {}
    for key, values in grouped.items():
        agg[key] = {}
        for metric in ["waiting_time", "execution_time", "response_time"]:
            means = np.array([v[metric]["mean"] for v in values])
            maxs  = np.array([v[metric]["max"]  for v in values])
            mins  = np.array([v[metric]["min"]  for v in values])
            
            agg[key][metric] = {
                "mean_of_means": np.mean(means),
                "std_of_means":  np.std(means),
                "mean_of_maxs":  np.mean(maxs),
                "std_of_maxs":   np.std(maxs),
                "mean_of_mins":  np.mean(mins),
                "std_of_mins":   np.std(mins),
            }
    
    return agg, grouped


def setup_subplots(n_rows, n_cols, figsize=None):
    """Crée une grille de subplots."""
    if figsize is None:
        figsize = (5 * n_cols, 4 * n_rows)
    fig, axes = plt.subplots(n_rows, n_cols, figsize=figsize, squeeze=False)
    return fig, axes.flatten()


def save_and_close(fig, filename):
    """Sauvegarde et ferme la figure."""
    filepath = os.path.join(RESULTS_DIR, filename)
    plt.tight_layout()
    plt.savefig(filepath, dpi=150, bbox_inches="tight")
    plt.close(fig)
    print(f"  ✓ Saved: {filepath}")


def hide_unused_axes(axes, n_used):
    """Masque les axes non utilisés."""
    for idx in range(n_used, len(axes)):
        axes[idx].set_visible(False)


def add_std_band(ax, x, y_mean, y_std, color, alpha=0.2):
    """Ajoute une bande d'incertitude basée sur l'écart-type."""
    y_upper = np.array(y_mean) + np.array(y_std)
    y_lower = np.array(y_mean) - np.array(y_std)
    y_lower = np.maximum(y_lower, 0)  # pas de valeurs négatives
    ax.fill_between(x, y_lower, y_upper, alpha=alpha, color=color)


# ============================================================
# PLOTS : MÉTRICS vs LENGTH
# ============================================================

def plot_metric_vs_length(agg, scenario_name, metric_key, ylabel):
    """
    Plot une métrique en fonction de la longueur, pour chaque nb_tasks.
    Layout: 2×2 si 4 plots, sinon adaptatif.
    """
    nb_tasks_list = sorted(set(k[2] for k in agg.keys() if k[0] == scenario_name))
    n_plots = len(nb_tasks_list)
    
  
    if n_plots == 4:
        n_rows, n_cols = 2, 2
    else:
        n_cols = min(n_plots, 3)
        n_rows = (n_plots + n_cols - 1) // n_cols
    
    fig, axes = setup_subplots(n_rows, n_cols)
    
    for idx, nb_tasks in enumerate(nb_tasks_list):
        ax = axes[idx]
        
        for io_time, cfg in IO_CONFIG.items():
            points = []
            for length in LENGTH_VALUES:
                key = (scenario_name, length, nb_tasks, io_time)
                if key not in agg:
                    continue
                
                stats = agg[key][metric_key]
                points.append({
                    "length": length,
                    "mean": stats["mean_of_means"],
                    "std":  stats["std_of_means"],
                })
            
            if not points:
                continue
            
            x = [p["length"] for p in points]
            y_mean = [p["mean"] for p in points]
            y_std  = [p["std"]  for p in points]
            
            ax.plot(x, y_mean, marker=cfg["marker"], linestyle=cfg["linestyle"],
                    color=cfg["color"], label=cfg["label"],
                    linewidth=2, markersize=7)
            add_std_band(ax, x, y_mean, y_std, cfg["color"])
        
        ax.set_xlabel("Task Length", fontsize=11)
        ax.set_ylabel(ylabel, fontsize=11)
        ax.set_title(f"{scenario_name.capitalize()} — {nb_tasks} tasks", fontsize=12, fontweight='bold')
        ax.legend(fontsize=9, loc='best')
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.set_xticks(LENGTH_VALUES)
    
    hide_unused_axes(axes, n_plots)
    save_and_close(fig, f"{scenario_name}_{metric_key}_vs_length.png")


# ============================================================
# PLOTS : MÉTRICS vs NB_TASKS
# ============================================================

def plot_metric_vs_nbtasks(agg, scenario_name, metric_key, ylabel, title):

    lengths = sorted(set(k[1] for k in agg.keys() if k[0] == scenario_name))
    n_plots = len(lengths)
    
    if n_plots == 4:
        n_rows, n_cols = 2, 2
    else:
        n_cols = min(n_plots, 3)
        n_rows = (n_plots + n_cols - 1) // n_cols
    
    fig, axes = setup_subplots(n_rows, n_cols)
    
    for idx, length in enumerate(lengths):
        ax = axes[idx]
        
        for io_time, cfg in IO_CONFIG.items():
            points = []
            for nb in NB_TASKS_VALUES:
                key = (scenario_name, length, nb, io_time)
                if key not in agg:
                    continue
                
                stats = agg[key][metric_key]
                points.append({
                    "nb": nb,
                    "mean": stats["mean_of_means"],
                    "std":  stats["std_of_means"],
                })
            
            if not points:
                continue
            
            x = [p["nb"] for p in points]
            y_mean = [p["mean"] for p in points]
            y_std  = [p["std"]  for p in points]
            
            ax.plot(x, y_mean, marker=cfg["marker"], linestyle=cfg["linestyle"],
                    color=cfg["color"], label=cfg["label"],
                    linewidth=2, markersize=7)
            add_std_band(ax, x, y_mean, y_std, cfg["color"])
        
        ax.set_xlabel("Number of Tasks", fontsize=11)
        ax.set_ylabel(ylabel, fontsize=11)
        ax.set_title(f"{title}\\nlength={length}", fontsize=12, fontweight='bold')
        ax.legend(fontsize=9, loc='best')
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.set_xticks(NB_TASKS_VALUES)
    
    hide_unused_axes(axes, n_plots)
    save_and_close(fig, f"{scenario_name}_{metric_key}_vs_nbtasks.png")

def plot_comparison_excluding_2048(agg, metric_key, ylabel):
    fig, axes = setup_subplots(2, 2, figsize=(12, 10))
    
    configs = [
        ("fix", 0),
        ("fix", 850),
        ("mix", 0),
        ("mix", 850),
    ]
    
    titles = [
        "Fix — IO=0ms",
        "Fix — IO=850ms",
        "Mix — IO=0ms",
        "Mix — IO=850ms",
    ]
    
    for idx, (scenario, io_time) in enumerate(configs):
        ax = axes[idx]
        
        all_lengths = sorted(set(k[1] for k in agg.keys() if k[0] == scenario))
        lengths = [l for l in all_lengths if l <1024]
        
        for length in lengths:
            points = []
            for nb in NB_TASKS_VALUES:
                key = (scenario, length, nb, io_time)
                if key not in agg:
                    continue
                stats = agg[key][metric_key]
                points.append({
                    "nb": nb,
                    "mean": stats["mean_of_means"] / 1000,  # ← Conversion en millisecondes
                    "std": stats["std_of_means"] / 1000,    # ← Conversion en millisecondes
                })
            
            if not points:
                continue
            
            x = [p["nb"] for p in points]
            y_mean = [p["mean"] for p in points]
            y_std  = [p["std"]  for p in points]
            
            color = plt.cm.viridis(lengths.index(length) / max(1, len(lengths) - 1))
            ax.plot(x, y_mean, marker="o", linestyle="-", color=color,
                    label=f"L={length}", linewidth=2, markersize=6)
            add_std_band(ax, x, y_mean, y_std, color)
        
        ax.set_xlabel("Number of Tasks", fontsize=11)
        ax.set_ylabel(f"{ylabel}", fontsize=11)  # ← Unité mise à jour
        ax.set_title(titles[idx], fontsize=12, fontweight='bold')
        ax.legend(fontsize=8, loc='best', title="Length")
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.set_xticks(NB_TASKS_VALUES)
    
    save_and_close(fig, f"comparison_{metric_key}_excluding_2048.png")
# ============================================================
# PLOTS : FAIRNESS & OVERHEAD
# ============================================================

def plot_fairness(agg, grouped):
 
    all_lengths = []
    for scenario in ["fix", "mix"]:
        lengths = sorted(set(k[1] for k in agg.keys() if k[0] == scenario))
        all_lengths.extend([(scenario, length) for length in lengths])
    
    n_plots = len(all_lengths)
    if n_plots == 4:
        n_rows, n_cols = 2, 2
    else:
        n_cols = min(n_plots, 3)
        n_rows = (n_plots + n_cols - 1) // n_cols
    
    fig, axes = setup_subplots(n_rows, n_cols)
    
    for plot_idx, (scenario_name, length) in enumerate(all_lengths):
        if plot_idx >= len(axes):
            break
        ax = axes[plot_idx]
        
        for io_time, cfg in IO_CONFIG.items():
            points = []
            for nb in NB_TASKS_VALUES:
                key = (scenario_name, length, nb, io_time)
                if key not in grouped:
                    continue
                
                fairness_per_run = [
                    v["response_time"]["max"] / v["response_time"]["mean"]
                    for v in grouped[key]
                ]
                
                points.append({
                    "nb": nb,
                    "mean": np.mean(fairness_per_run),
                    "std": np.std(fairness_per_run),
                })
            
            if not points:
                continue
            
            x = [p["nb"] for p in points]
            y = [p["mean"] for p in points]
            y_err = [p["std"] for p in points]
            
            ax.plot(x, y, marker=cfg["marker"], linestyle=cfg["linestyle"],
                    color=cfg["color"], label=cfg["label"],
                    linewidth=2, markersize=7)
            add_std_band(ax, x, y, y_err, cfg["color"])
        
        ax.set_xlabel("Number of Tasks", fontsize=11)
        ax.set_ylabel("Fairness (max / mean)", fontsize=11)
        ax.set_title(f"{scenario_name.capitalize()} Fairness — length={length}",
                     fontsize=12, fontweight='bold')
        ax.legend(fontsize=9, loc='best')
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.axhline(y=1.0, color="red", linestyle=":", alpha=0.6, linewidth=1.5)
        ax.set_xticks(NB_TASKS_VALUES)
    
    hide_unused_axes(axes, len(all_lengths))
    save_and_close(fig, "fairness.png")


def plot_overhead(agg, grouped):

    all_lengths = []
    for scenario in ["fix", "mix"]:
        lengths = sorted(set(k[1] for k in agg.keys() if k[0] == scenario))
        all_lengths.extend([(scenario, length) for length in lengths])
    
    n_plots = len(all_lengths)
    if n_plots == 4:
        n_rows, n_cols = 2, 2
    else:
        n_cols = min(n_plots, 3)
        n_rows = (n_plots + n_cols - 1) // n_cols
    
    fig, axes = setup_subplots(n_rows, n_cols)
    
    for plot_idx, (scenario_name, length) in enumerate(all_lengths):
        if plot_idx >= len(axes):
            break
        ax = axes[plot_idx]
        
        for io_time, cfg in IO_CONFIG.items():
            points = []
            for nb in NB_TASKS_VALUES:
                key = (scenario_name, length, nb, io_time)
                if key not in grouped:
                    continue
                
                overhead_per_run = [
                    v["waiting_time"]["mean"] / v["response_time"]["mean"]
                    for v in grouped[key]
                ]
                
                points.append({
                    "nb": nb,
                    "mean": np.mean(overhead_per_run),
                    "std": np.std(overhead_per_run),
                })
            
            if not points:
                continue
            
            x = [p["nb"] for p in points]
            y = [p["mean"] for p in points]
            y_err = [p["std"] for p in points]
            
            ax.plot(x, y, marker=cfg["marker"], linestyle=cfg["linestyle"],
                    color=cfg["color"], label=cfg["label"],
                    linewidth=2, markersize=7)
            add_std_band(ax, x, y, y_err, cfg["color"])
        
        ax.set_xlabel("Number of Tasks", fontsize=11)
        ax.set_ylabel("Overhead (waiting / response)", fontsize=11)
        ax.set_title(f"{scenario_name.capitalize()} Overhead — length={length}",
                     fontsize=12, fontweight='bold')
        ax.legend(fontsize=9, loc='best')
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.set_xticks(NB_TASKS_VALUES)
    
    hide_unused_axes(axes, len(all_lengths))
    save_and_close(fig, "overhead.png")


# ============================================================
# PLOT COMPARATIF : FIX vs MIX
# ============================================================

def plot_comparison(agg, metric_key, ylabel):

    fig, axes = setup_subplots(2, 2, figsize=(12, 10))
    

    
    configs = [
        ("fix", 0),
        ("fix", 850),
        ("mix", 0),
        ("mix", 850),
    ]
    
    titles = [
        "Fix — IO=0ms",
        "Fix — IO=850ms",
        "Mix — IO=0ms",
        "Mix — IO=850ms",
    ]
    
    for idx, (scenario, io_time) in enumerate(configs):
        ax = axes[idx]
        
        lengths = sorted(set(k[1] for k in agg.keys() if k[0] == scenario))
        
        for length in lengths:
            points = []
            for nb in NB_TASKS_VALUES:
                key = (scenario, length, nb, io_time)
                if key not in agg:
                    continue
                stats = agg[key][metric_key]
                points.append({
                    "nb": nb,
                    "mean": stats["mean_of_means"],
                    "std": stats["std_of_means"],
                })
            
            if not points:
                continue
            
            x = [p["nb"] for p in points]
            y_mean = [p["mean"] for p in points]
            y_std  = [p["std"]  for p in points]
            
            color = plt.cm.viridis(lengths.index(length) / max(1, len(lengths) - 1))
            ax.plot(x, y_mean, marker="o", linestyle="-", color=color,
                    label=f"L={length}", linewidth=2, markersize=6)
            add_std_band(ax, x, y_mean, y_std, color)
        
        ax.set_xlabel("Number of Tasks", fontsize=11)
        ax.set_ylabel(ylabel, fontsize=11)
        ax.set_title(titles[idx], fontsize=12, fontweight='bold')
        ax.legend(fontsize=8, loc='best', title="Length")
        ax.grid(True, alpha=0.3, linestyle='--')
        ax.set_xticks(NB_TASKS_VALUES)
    
    save_and_close(fig, f"comparison_{metric_key}.png")


# ============================================================
# MAIN
# ============================================================

def main():
    os.makedirs(RESULTS_DIR, exist_ok=True)
    
    data = load_data(INPUT_FILE)
    agg, grouped = aggregate_runs(data)

    print(f"\\n data charged: {len(data)} runs")
  
    print("[1/5] Plots: Métrics vs Task Length")
    for scenario in ["fix", "mix"]:
        for metric, ylabel in [
            ("execution_time", "Execution Time (µs)"),
            ("waiting_time",   "Waiting Time (µs)"),
            ("response_time",  "Response Time (µs)"),
        ]:
            plot_metric_vs_length(agg, scenario, metric, ylabel)
    
   
    print("\\n[2/5] Plots: Métrics vs Number of Tasks (par length)")
    for scenario in ["fix", "mix"]:
        for metric, ylabel, title_suffix in [
            ("waiting_time",   "µs", "Waiting Time"),
            ("execution_time", "µs", "Execution Time"),
            ("response_time",  "µs", "Response Time"),
        ]:
            plot_metric_vs_nbtasks(agg, scenario, metric, ylabel,
                                   f"{scenario.capitalize()} {title_suffix}")
    

    print("\\n[3/5] Plot: Fairness")
    plot_fairness(agg, grouped)
    
    print("\\n[4/5] Plot: Overhead")
    plot_overhead(agg, grouped)
    

    print("\\n[5/5] Plots: Comparatifs Fix vs Mix")
    for metric, ylabel in [
        ("execution_time", "Execution Time (ms)"),
        ("response_time",  "Response Time (ms)"),
    ]:
        plot_comparison_excluding_2048(agg, metric, ylabel)
        plot_comparison(agg,metric,ylabel)
    print("done")

if __name__ == "__main__":
    main()
