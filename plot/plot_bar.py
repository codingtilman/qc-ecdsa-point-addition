"""Bar chart comparing two versions.

Usage: python plot_bar.py [A] [B] [--out FILE] [--title TEXT]
Default: Baseline vs Official87 -> research_bar.png
"""
import argparse
import csv
import math
from pathlib import Path

import matplotlib.pyplot as plt

HERE = Path(__file__).parent
N = 100_000

ap = argparse.ArgumentParser()
ap.add_argument("versions", nargs="*", default=["Baseline", "Official87"])
ap.add_argument("--out", default="research_bar.png")
ap.add_argument("--title", default="Best version vs. baseline")
ap.add_argument("--labels", nargs="*", help="display names for the versions (use \\n for a line break)")
args = ap.parse_args()
KEEP = args.versions

table = {r["version"]: r for r in csv.DictReader(open(HERE / "research_results.csv"))}
rows = [table[k] for k in KEEP]
score = [int(r["score"]) / 1e6 for r in rows]
fails = [int(r["union_failures_per_100000"]) for r in rows]
err = [k / N * 100 for k in fails]
passed = [r["official_local_pass"] == "True" for r in rows]


def wilson(k, n, z=1.96):
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (c - h) * 100, (c + h) * 100


ci = [wilson(k, N) for k in fails]
yerr = [[e - lo for e, (lo, _) in zip(err, ci)], [hi - e for e, (_, hi) in zip(err, ci)]]

INK, MUTED, GRID = "#1f2933", "#5b6573", "#e6e9ee"
PALETTE = {"Baseline": "#9aa5b4", "Official87": "#0f8a6c",
           "E052": "#e08a12", "E056": "#7b52c4"}
COLORS = [PALETTE.get(v, "#9aa5b4") for v in KEEP]

plt.rcParams.update({
    "font.family": "DejaVu Sans", "font.size": 11,
    "axes.spines.top": False, "axes.spines.right": False, "axes.spines.left": False,
    "axes.edgecolor": "#b8c0cc", "xtick.color": INK, "ytick.color": MUTED,
    "axes.grid": True, "axes.grid.axis": "y", "grid.color": GRID,
})

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4.8))
fig.subplots_adjust(left=0.07, right=0.97, top=0.80, bottom=0.13, wspace=0.3)
x = [0, 1]


def delta(a, b, unit, fmt):
    sign = "−" if b < a else "+"
    return f"{sign}{fmt(abs(b - a))}{unit}  ({sign}{abs(b - a) / a * 100:.4g}%)"


# ---- Score ----
b = ax1.bar(x, score, width=0.55, color=COLORS, zorder=3)
ax1.bar_label(b, labels=[f"{int(r['score']):,}" for r in rows], padding=4,
              fontsize=11.5, fontweight="bold", color=INK)
ax1.set_ylim(0, max(score) * 1.18)
diff = int(rows[1]["score"]) - int(rows[0]["score"])
ax1.text(0.5, max(score) * 1.11,
         f"{'−' if diff < 0 else '+'}{abs(diff):,}  "
         f"({'−' if diff < 0 else '+'}{abs(diff) / int(rows[0]['score']) * 100:.4f}%)",
         ha="center", fontsize=11, color=COLORS[1], fontweight="bold")
ax1.set_title("Score (millions)", loc="left", fontsize=13, color=INK, pad=12)

# ---- Error rate ----
ax2.bar(x, err, width=0.55, color=COLORS, zorder=3)
ax2.errorbar(x, err, yerr=yerr, fmt="none", ecolor=INK, elinewidth=1.2,
             capsize=6, zorder=4)
for xi, e, (_, hi) in zip(x, err, ci):
    ax2.text(xi, hi + 0.008, f"{e:.3f}%", ha="center", va="bottom",
             fontsize=12, fontweight="bold", color=INK)
ax2.set_ylim(0, 0.32)
ax2.set_title("Error rate (%)", loc="left", fontsize=13, color=INK, pad=12)

for ax in (ax1, ax2):
    names = [s.replace("\\n", "\n") for s in args.labels] if args.labels else KEEP
    ax.set_xticks(x, names, fontsize=12, linespacing=1.4)
    ax.tick_params(axis="x", length=0)
    ax.tick_params(axis="y", length=0)
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda v, _: f"{v:g}"))

fig.suptitle(args.title, x=0.07, y=0.97, ha="left",
             fontsize=17, fontweight="bold", color=INK)

fig.savefig(HERE / args.out, dpi=200, facecolor="white")
