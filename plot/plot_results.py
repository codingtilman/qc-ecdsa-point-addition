import csv
import math
from pathlib import Path

import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

HERE = Path(__file__).parent
N = 100_000

rows = list(csv.DictReader(open(HERE / "research_results.csv")))
for r in rows:
    r["x"] = int(r["score"]) / 1e6
    r["k"] = int(r["union_failures_per_100000"])
    r["y"] = r["k"] / N * 100
    r["ok"] = r["official_local_pass"] == "True"


def wilson(k, n, z=1.96):
    p = k / n
    d = 1 + z * z / n
    c = (p + z * z / (2 * n)) / d
    h = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / d
    return (c - h) * 100, (c + h) * 100


INK, MUTED, GRID = "#1f2933", "#8a94a3", "#e6e9ee"
COLORS = {"Official87": "#0f8a6c", "E052": "#e08a12", "E056": "#7b52c4",
          "Baseline": "#3d4b5c"}
LEADERS = ["E052", "E056", "Official87"]

plt.rcParams.update({
    "font.family": "DejaVu Sans", "font.size": 11,
    "axes.spines.top": False, "axes.spines.right": False,
    "axes.edgecolor": "#b8c0cc", "axes.labelcolor": INK,
    "xtick.color": "#5b6573", "ytick.color": "#5b6573",
    "axes.grid": True, "grid.color": GRID, "grid.linewidth": 0.8,
})

fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(13, 6.6),
                               gridspec_kw={"width_ratios": [1.15, 1]})
fig.subplots_adjust(left=0.06, right=0.98, top=0.835, bottom=0.30, wspace=0.18)


def draw(ax, r, size):
    c = COLORS.get(r["version"], MUTED)
    ax.scatter(r["x"], r["y"], s=size * (1.9 if r["ok"] else 1),
               marker="*" if r["ok"] else "o", color=c,
               edgecolor="white", linewidth=1.2, zorder=4)
    return c


# ---- Overview ----
for r in rows:
    draw(ax1, r, 120)
for name, dx, dy, ha in [("Baseline", 0, 0.12, "center"),
                         ("E040", 0.15, 0.08, "left"),
                         ("E046", 0.15, 0, "left")]:
    r = next(r for r in rows if r["version"] == name)
    ax1.annotate(name, (r["x"], r["y"]), (r["x"] + dx, r["y"] + dy),
                 ha=ha, va="center", color=COLORS.get(name, MUTED),
                 fontsize=11, fontweight="bold")

box = Rectangle((890.55, 0.05), 0.75, 0.32, fill=False, ec=INK, lw=1,
                ls=(0, (3, 2)), zorder=3)
ax1.add_patch(box)
ax1.annotate("Leaders", (891.3, 0.30), (891.5, 0.55), color=INK,
             fontsize=11, fontweight="bold",
             arrowprops=dict(arrowstyle="-", color=INK, lw=0.8))
ax1.set_xlim(890.2, 897.2)
ax1.set_ylim(0, 2.1)
ax1.set_xlabel("Score (millions)")
ax1.set_ylabel("Error rate (%)")
ax1.set_title("All versions", loc="left", fontsize=13, color=INK, pad=10)

# ---- Zoom on leaders ----
for name in LEADERS:
    r = next(r for r in rows if r["version"] == name)
    lo, hi = wilson(r["k"], N)
    c = COLORS[name]
    ax2.plot([r["x"]] * 2, [lo, hi], color=c, lw=6, alpha=0.18,
             solid_capstyle="round", zorder=2)
    draw(ax2, r, 200)
    ax2.text(r["x"], hi + 0.006, name, ha="center", va="bottom",
             color=c, fontsize=12, fontweight="bold")
    ax2.text(r["x"], lo - 0.006, f"{r['y']:.3f}%", ha="center", va="top",
             color=c, fontsize=10)
ax2.set_xlim(890.890, 890.925)
ax2.set_ylim(0.16, 0.27)
ax2.xaxis.set_major_formatter(plt.FuncFormatter(lambda v, _: f"{v:.2f}"))
ax2.set_xticks([890.89, 890.90, 890.91, 890.92])
ax2.set_xlabel("Score (millions)")
ax2.set_title("Leaders  ·  95% CI", loc="left", fontsize=13, color=INK, pad=10)

for ax in (ax1, ax2):
    ax.yaxis.set_major_formatter(plt.FuncFormatter(lambda v, _: f"{v:g}"))

fig.suptitle("secp256k1 point-add: cost vs. error", x=0.06, y=0.985, ha="left",
             fontsize=17, fontweight="bold", color=INK)
fig.text(0.06, 0.905, "Lower-left is better   ★ passed official check   ● failed",
         fontsize=11, color="#5b6573")

CAPTION = [
    ("Score", "peak qubits × mean Toffoli count, in millions. Lower = cheaper circuit."),
    ("Error rate", "share of 100,000 fixed test inputs that fail (wrong value, phase or ancilla)."),
    ("★ / ●", "passed / failed the local official 9,024-shot check."),
    ("Right panel", "zoom on the dashed box; bars = 95% CI. Bars overlap, so E052/E056 are cheaper"
                    " than Official87 but not clearly more accurate."),
]
fig.add_artist(plt.Line2D([0.06, 0.98], [0.185, 0.185], color=GRID, lw=1))
y = 0.16
for head, body in CAPTION:
    fig.text(0.06, y, head, fontsize=10, color=INK, fontweight="bold", va="top")
    fig.text(0.155, y, body, fontsize=10, color="#5b6573", va="top", linespacing=1.4)
    y -= 0.03 * (body.count("\n") + 1) + 0.005
fig.text(0.98, 0.012, "Source: research checkpoints, 2026-11-06", ha="right",
         fontsize=8.5, color=MUTED)

fig.savefig(HERE / "research_results.png", dpi=200, facecolor="white")
