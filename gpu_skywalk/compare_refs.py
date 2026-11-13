#!/usr/bin/env python3
import struct
import sys

def load(path):
    with open(path, "rb") as f:
        assert f.read(8) == b"SKYGPU1\0"
        h = struct.unpack("<10Q", f.read(80))
        version, nops, nq, nb, shots, batches, nmeas, nout, ndecl, depth = h
        assert version == 1 and depth <= 4
        skip = nops * 16 + (nout + ndecl) * 2
        skip += batches * (nq + nb) * 8
        skip += batches * nout * 8
        skip += batches * 208
        f.seek(skip, 1)
        refs = [struct.unpack("<4Q", f.read(32)) for _ in range(batches)]
        assert not f.read(1)
    return shots, refs

def pc(x): return x.bit_count()
def counts(rs, col): return sum(pc(r[col]) for r in rs)

def main(a, b):
    na, ra = load(a); nb, rb = load(b)
    assert na == nb and len(ra) == len(rb)
    names = ["value", "phase", "anc"]
    for j, name in enumerate(names):
        ao = bo = 0
        for x, y in zip(ra, rb):
            ao += pc(x[j] & ~y[j]); bo += pc(y[j] & ~x[j])
        print(f"{name} a={counts(ra,j)} b={counts(rb,j)} a_only={ao} b_only={bo}")
    aa = [r[0] | r[1] | r[2] for r in ra]
    bb = [r[0] | r[1] | r[2] for r in rb]
    ao = sum(pc(x & ~y) for x,y in zip(aa,bb))
    bo = sum(pc(y & ~x) for x,y in zip(aa,bb))
    print(f"any a={sum(map(pc,aa))} b={sum(map(pc,bb))} a_only={ao} b_only={bo}")
    ta = sum(r[3] for r in ra); tb = sum(r[3] for r in rb)
    print(f"toffoli a={ta} b={tb} mean_a={ta/na:.3f} mean_b={tb/nb:.3f}")

if __name__ == "__main__":
    assert len(sys.argv) == 3, "usage: compare_refs.py A.pack B.pack"
    main(sys.argv[1], sys.argv[2])
