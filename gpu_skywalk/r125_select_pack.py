#!/usr/bin/env python3
"""Select nonce-sized batch groups from a v2 R125 pack for an exact next stage."""

import argparse
import mmap
import struct


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("source_pack")
    ap.add_argument("partial_masks")
    ap.add_argument("output_pack")
    ap.add_argument("count", type=int)
    ap.add_argument("partial_shots", type=int)
    args = ap.parse_args()
    pbn, fbn = args.partial_shots // 64, 9024 // 64
    masks = list(struct.iter_unpack("<QQQQ", open(args.partial_masks, "rb").read()))
    keep = []
    for i in range(args.count):
        part = masks[i * pbn:(i + 1) * pbn]
        if not any(v | ph | a for v, ph, a, _ in part):
            keep.append(i)
    with open(args.source_pack, "rb") as src, open(args.output_pack, "wb") as out:
        mm = mmap.mmap(src.fileno(), 0, access=mmap.ACCESS_READ)
        if mm[:8] != b"SKYGPU1\0":
            raise SystemExit("bad pack")
        h = list(struct.unpack_from("<10Q", mm, 8))
        version, nops, nq, nb, shots, batches, nmeas, nout, ndecl, depth = h
        if version != 2 or shots != args.count * 9024 or batches != args.count * fbn:
            raise SystemExit("source is not the expected full v2 nonce pack")
        new_batches = len(keep) * fbn
        new_h = [version, nops, nq, nb, len(keep) * 9024, new_batches, nmeas, nout, ndecl, depth]
        out.write(mm[:8]); out.write(struct.pack("<10Q", *new_h))
        pos = 88
        constant = nops * 16 + (nout + ndecl) * 2
        out.write(mm[pos:pos + constant]); pos += constant
        sizes = [(nq + nb) * 8, nout * 8, 26 * 8]
        section_rows = batches
        for row_size in sizes:
            base = pos
            for i in keep:
                lo = base + i * fbn * row_size
                out.write(mm[lo:lo + fbn * row_size])
            pos += section_rows * row_size
        if pos != len(mm):
            raise SystemExit(f"unexpected trailing bytes: {len(mm)-pos}")
        mm.close()
    print(f"SELECT input_nonces={args.count} kept={len(keep)} indices={','.join(map(str,keep))}")


if __name__ == "__main__":
    main()
