#!/usr/bin/env python3
"""Reduce architecture-matched R125 GPU masks into per-nonce verdicts."""

import argparse
import struct


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("masks")
    ap.add_argument("start", type=int)
    ap.add_argument("count", type=int)
    ap.add_argument("--shots", type=int, default=9024)
    args = ap.parse_args()
    data = open(args.masks, "rb").read()
    rows = list(struct.iter_unpack("<QQQQ", data))
    if args.shots <= 0 or args.shots % 64:
        raise SystemExit("--shots must be a positive multiple of 64")
    batches = args.shots // 64
    if len(rows) != args.count * batches:
        raise SystemExit(f"got {len(rows)} batches, expected {args.count * batches}")
    clean = []
    for j in range(args.count):
        part = rows[j * batches:(j + 1) * batches]
        v = sum(r[0].bit_count() for r in part)
        p = sum(r[1].bit_count() for r in part)
        a = sum(r[2].bit_count() for r in part)
        u = sum((r[0] | r[1] | r[2]).bit_count() for r in part)
        nonce = args.start + j
        print(f"NONCE nonce={nonce} value={v} phase={p} ancilla={a} union={u}")
        if u == 0:
            clean.append(nonce)
            print(f"CLEAN nonce={nonce}")
    print(f"SUMMARY start={args.start} count={args.count} shots={args.shots} clean={len(clean)}")


if __name__ == "__main__":
    main()
