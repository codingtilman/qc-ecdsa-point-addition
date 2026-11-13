#!/usr/bin/env python3
"""Compare raw Skywalk GPU Result masks on an identical ordered corpus."""

import math
import struct
import sys


def load(path):
    data = open(path, "rb").read()
    if len(data) % 32:
        raise SystemExit(f"{path}: invalid byte count {len(data)}")
    return list(struct.iter_unpack("<QQQQ", data))


def bits(words, field):
    return sum(row[field].bit_count() for row in words)


def wilson(k, n, z=1.959963984540054):
    p = k / n
    den = 1 + z * z / n
    mid = (p + z * z / (2 * n)) / den
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / den
    return mid - half, mid + half


def union_rows(rows):
    return [v | p | a for v, p, a, _ in rows]


def main():
    if len(sys.argv) < 4 or len(sys.argv) % 2 != 0:
        raise SystemExit("usage: compare_masks.py SHOTS LABEL FILE [LABEL FILE ...]")
    shots = int(sys.argv[1])
    series = [(sys.argv[i], load(sys.argv[i + 1])) for i in range(2, len(sys.argv), 2)]
    batches = (shots + 63) // 64
    for label, rows in series:
        if len(rows) != batches:
            raise SystemExit(f"{label}: got {len(rows)} records, expected {batches}")
        v, p, a = (bits(rows, i) for i in range(3))
        u = sum(x.bit_count() for x in union_rows(rows))
        tof = sum(row[3] for row in rows)
        lo, hi = wilson(u, shots)
        print(f"TOTAL {label} value={v} phase={p} anc={a} union={u} "
              f"rate={u/shots:.8f} ci95=[{lo:.8f},{hi:.8f}] T_mean={tof/shots:.6f}")
    for i, (la, ra) in enumerate(series):
        ua = union_rows(ra)
        for lb, rb in series[i + 1:]:
            ub = union_rows(rb)
            a_only = sum((a & ~b).bit_count() for a, b in zip(ua, ub))
            b_only = sum((b & ~a).bit_count() for a, b in zip(ua, ub))
            both = sum((a & b).bit_count() for a, b in zip(ua, ub))
            delta = (b_only - a_only) / shots
            se = math.sqrt((a_only + b_only) / shots**2 - delta**2 / shots)
            print(f"PAIR {la} {lb} {la}_only={a_only} {lb}_only={b_only} both={both} "
                  f"discordant={a_only+b_only} delta={delta:.8f} "
                  f"delta_ci95=[{delta-1.959963984540054*se:.8f},{delta+1.959963984540054*se:.8f}]")


if __name__ == "__main__":
    main()
