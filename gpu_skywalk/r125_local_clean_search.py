#!/usr/bin/env python3
"""Bounded local-only R125 clean-nonce search.

Approximate filters only rank the queue. Every reported survivor is replayed by
the architecture-matched exact GPU circuit before it can be reported clean.
No submission or network action exists in this program.
"""

import argparse
import json
import os
import re
import struct
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "gpu_toolkit/gpu_island2_r125_fold"
STATE = Path("/tmp/r125-e048-gcd402.gpu_state")
PACKER = ROOT / "gpu_skywalk/target/release/skywalk_gpu_pack"
REPLAY = ROOT / "gpu_skywalk/skywalk_replay"
OPS = ROOT / "e048_rebuild/ops.bin"


def run_logged(cmd, env, log):
    started = time.monotonic()
    with log.open("w") as f:
        p = subprocess.run(cmd, env=env, stdout=f, stderr=subprocess.STDOUT)
    if p.returncode:
        raise RuntimeError(f"command failed ({p.returncode}); see {log}")
    return time.monotonic() - started


def filter_passes(start, count, mode, outdir, fold_window=56):
    tag = f"{mode}-{start}-{count}"
    log = outdir / f"{tag}.log"
    env = os.environ.copy()
    env.update({
        "GPU_STATE": str(STATE), "GPU_FILTER": mode,
        "R125_SCREEN_SHOTS": "9024", "KERNEL2": "1", "BLOCKS": "512",
        "GPU_BATCH_INV": "1", "GPU_COMB_BITS": "22", "GPU_WAVE": "128",
    })
    if mode == "r125_fold":
        env["R125_FOLD_WINDOW"] = str(fold_window)
    else:
        env["R125_GCD_LIMIT"] = "261"
    elapsed = run_logged([str(TOOL), str(start), str(count)], env, log)
    text = log.read_text()
    passed = {int(x) for x in re.findall(r"CLEAN nonce=(\d+)", text)}
    return passed, elapsed, text.splitlines()[-1]


def exact_stage(nonces, shots, outdir, label):
    pack = outdir / f"{label}-s{shots}.pack"
    masks = outdir / f"{label}-s{shots}.masks"
    plog = outdir / f"{label}-s{shots}-pack.log"
    glog = outdir / f"{label}-s{shots}-gpu.log"
    env = os.environ.copy()
    env.update({
        "SKYWALK_NONCE_LIST": ",".join(map(str, nonces)),
        "SKYWALK_NONCE_SHOTS": str(shots),
        "SKYWALK_GPU_SCREEN": "1", "SKYWALK_INPUT_THREADS": "16",
    })
    pt = run_logged([str(PACKER), str(OPS), str(len(nonces) * shots), str(pack)], env, plog)
    env = os.environ.copy()
    env.update({
        "SKYWALK_SCREEN_ONLY": "1", "SKYWALK_EARLY_BATCHES_PER_NONCE": "1",
        "SKYWALK_MASK_OUT": str(masks),
    })
    gt = run_logged([str(REPLAY), str(pack)], env, glog)
    pack.unlink()  # Regenerable multi-hundred-MiB transport; logs and masks remain.
    rows = list(struct.iter_unpack("<QQQQ", masks.read_bytes()))
    batches = shots // 64
    assert len(rows) == len(nonces) * batches
    passed = []
    for j, nonce in enumerate(nonces):
        part = rows[j * batches:(j + 1) * batches]
        if all((v | p | a) == 0 for v, p, a, _ in part):
            passed.append(nonce)
    return passed, pt, gt


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--start", type=int, required=True)
    ap.add_argument("--count", type=int, default=65536)
    ap.add_argument("--chunk", type=int, default=4096)
    ap.add_argument("--seconds", type=int, default=1500)
    ap.add_argument("--out", type=Path, required=True)
    args = ap.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    deadline = time.monotonic() + args.seconds
    tested = 0
    records = []
    clean = []
    while tested < args.count and time.monotonic() < deadline:
        start = args.start + tested
        count = min(args.chunk, args.count - tested)
        fold, ft, fs = filter_passes(start, count, "r125_fold", args.out)
        gcd, gt, gs = filter_passes(start, count, "dialog_gcd", args.out)
        queue = sorted(fold & gcd)
        rec = {"start": start, "count": count, "fold_pass": len(fold),
               "gcd_pass": len(gcd), "intersection": len(queue),
               "fold_seconds": ft, "gcd_seconds": gt,
               "fold_summary": fs, "gcd_summary": gs, "stages": []}
        for shots in (512, 1024, 2048, 9024):
            if not queue:
                break
            before = len(queue)
            queue, pt, et = exact_stage(queue, shots, args.out, f"n{start}")
            rec["stages"].append({"shots": shots, "input": before,
                                  "pass": len(queue), "pack_seconds": pt,
                                  "gpu_seconds": et})
        if queue:
            clean.extend(queue)
        tested += count
        rec["elapsed_total"] = args.seconds - max(0, deadline - time.monotonic())
        records.append(rec)
        checkpoint = {"candidate": "E048 bdeeee93", "ops": str(OPS),
                      "start": args.start, "tested": tested,
                      "next": args.start + tested, "clean_gpu_exact": clean,
                      "records": records}
        (args.out / "checkpoint.json").write_text(json.dumps(checkpoint, indent=2) + "\n")
        print("CHUNK", json.dumps(rec, separators=(",", ":")), flush=True)
        if clean:
            break
    print(json.dumps({"tested": tested, "next": args.start + tested,
                      "clean_gpu_exact": clean,
                      "wall_seconds": args.seconds - max(0, deadline - time.monotonic())}),
          flush=True)


if __name__ == "__main__":
    main()
