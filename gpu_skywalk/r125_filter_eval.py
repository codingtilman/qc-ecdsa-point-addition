#!/usr/bin/env python3
"""Compare an R125 partial-shot mask with full 9,024-shot ground truth."""

import argparse
import struct


def rows(path):
    data = open(path, "rb").read()
    if len(data) % 32:
        raise SystemExit(f"invalid mask length: {path}")
    return list(struct.iter_unpack("<QQQQ", data))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("partial")
    ap.add_argument("full")
    ap.add_argument("count", type=int)
    ap.add_argument("shots", type=int)
    args = ap.parse_args()
    if args.shots % 64:
        raise SystemExit("shots must be a multiple of 64")
    pbn, fbn = args.shots // 64, 9024 // 64
    part, full = rows(args.partial), rows(args.full)
    if len(part) != args.count * pbn or len(full) != args.count * fbn:
        raise SystemExit("mask/count mismatch")
    prefix_mismatch = hard_reject = uncertain_dirty = clean_total = clean_rejected = 0
    for i in range(args.count):
        pr = part[i * pbn:(i + 1) * pbn]
        fr = full[i * fbn:(i + 1) * fbn]
        prefix_mismatch += sum(a[:3] != b[:3] for a, b in zip(pr, fr[:pbn]))
        pbad = any((v | ph | a) != 0 for v, ph, a, _ in pr)
        fbad = any((v | ph | a) != 0 for v, ph, a, _ in fr)
        if pbad:
            hard_reject += 1
            if not fbad:
                clean_rejected += 1
        elif fbad:
            uncertain_dirty += 1
        else:
            clean_total += 1
    dirty_total = args.count - clean_total
    print(f"FILTER shots={args.shots} nonces={args.count} prefix_mask_mismatch={prefix_mismatch}")
    print(f"CONFUSION full_dirty={dirty_total} full_clean={clean_total} hard_reject={hard_reject} "
          f"uncertain_dirty={uncertain_dirty} clean_rejected={clean_rejected}")
    recall = hard_reject / dirty_total if dirty_total else float('nan')
    pass_fraction = (args.count - hard_reject) / args.count
    print(f"METRICS dirty_reject_recall={recall:.8f} pass_fraction={pass_fraction:.8f} "
          f"observed_clean_recall={(clean_total-clean_rejected)/clean_total if clean_total else float('nan'):.8f}")


if __name__ == "__main__":
    main()
