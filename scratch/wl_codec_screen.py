#!/usr/bin/env python3
"""Exact classical-support screen for ordinary WL codec Toffolis.

Enumerates the documented 26^4 legal input words as bit-sliced Python ints,
executes the generated ordinary commit then pop, and reports CCX controls that
are constant or equal to a current affine wire expression.  HMR resets its
measured qubit; conditional Z/CZ/CCZ operations do not change basis values.
"""

from itertools import product
from pathlib import Path
import re

SRC = Path("/workspace/project/ecdsa-research/baseline/src/point_add/wl_codec.rs")


def encode(k3, k2):
    f0, f1 = k3[0], k3[1]
    c1, c2, c3 = k2
    m = f1 & f0
    c1 ^= m
    f1 ^= m
    q = k3[2]
    x1 = q & f0
    x2 = q & f1
    c3 ^= x1
    c2 ^= x2
    f0 ^= q ^ x2 ^ x1
    f1 ^= q ^ x2
    return f0, f1, c1, c2, c3


def legal_words():
    out = []
    for k3 in product((0, 1), repeat=3):
        if sum(k3) > 2:
            continue
        free = [i for i in range(3) if not k3[i]]
        for vals in product((0, 1), repeat=len(free)):
            k2 = [0, 0, 0]
            for i, v in zip(free, vals):
                k2[i] = v
            out.append(encode(k3, k2))
    assert len(out) == len(set(out)) == 26
    return out


def parse_ops(name):
    text = SRC.read_text()
    m = re.search(rf"const {name}: &\[WlOp\] = &\[(.*?)\];", text, re.S)
    assert m, name
    return [(kind, tuple(map(int, args.split(","))))
            for kind, args in re.findall(r"WlOp::(\w+)\(([^)]*)\)", m.group(1))]


words = legal_words()
n = 26 ** 4
mask = (1 << n) - 1
wires = [0] * 23
for block in range(4):
    stride = 26 ** (3 - block)
    for bit in range(5):
        value = 0
        for row in range(n):
            wi = (row // stride) % 26
            if words[wi][bit]:
                value |= 1 << row
        wires[5 * block + bit] = value


def affine_single_or_pair(value, target):
    choices = [(mask, "1")]
    choices += [(v, f"w{i}") for i, v in enumerate(wires) if i != target]
    for v, name in choices:
        if value == v:
            return name
        if value == (v ^ mask):
            return f"1^{name}"
    for i in range(len(choices)):
        for j in range(i + 1, len(choices)):
            if value == (choices[i][0] ^ choices[j][0]):
                return f"{choices[i][1]}^{choices[j][1]}"
    return None


def affine_span(value, target):
    basis = {}
    vectors = [(mask, "1")] + [(v, f"w{i}") for i, v in enumerate(wires) if i != target]
    for value0, name in vectors:
        v, names = value0, [name]
        while v:
            p = v.bit_length() - 1
            if p not in basis:
                basis[p] = (v, names)
                break
            v ^= basis[p][0]
            names += basis[p][1]
    v, names = value, []
    while v:
        p = v.bit_length() - 1
        if p not in basis:
            return None
        v ^= basis[p][0]
        names += basis[p][1]
    # Cancel duplicate recipe names over GF(2).
    return "^".join(sorted(name for name in set(names) if names.count(name) & 1)) or "0"


hits = []


def run(label, ops):
    for oi, (kind, a) in enumerate(ops):
        if kind == "X":
            wires[a[0]] ^= mask
        elif kind == "Cx":
            wires[a[1]] ^= wires[a[0]]
        elif kind == "Ccx":
            p = wires[a[0]] & wires[a[1]]
            aff = affine_single_or_pair(p, a[2]) or affine_span(p, a[2])
            count = p.bit_count()
            if count == 0 or aff is not None:
                hits.append((label, oi, a, count, aff))
            wires[a[2]] ^= p
        elif kind == "Swap":
            wires[a[0]], wires[a[1]] = wires[a[1]], wires[a[0]]
        elif kind == "Alloc":
            assert wires[a[0]] == 0
        elif kind == "Free":
            assert wires[a[0]] == 0, (label, oi, a, wires[a[0]].bit_count())
            wires[a[0]] = 0
        elif kind == "Hmr":
            wires[a[0]] = 0
        elif kind in ("Zif", "Czif", "Cczif"):
            pass
        else:
            raise AssertionError((label, oi, kind, a))


initial = wires[:20].copy()
run("commit", parse_ops("ORDINARY_COMMIT"))
assert wires[4] == 0
run("pop", parse_ops("ORDINARY_POP"))
assert wires[:20] == initial
assert all(v == 0 for v in wires[20:])
print(f"ordinary states={n} exact commit/pop roundtrip=yes")
print(f"constant/affine CCX candidates={len(hits)}")
for hit in hits:
    print(hit)


def prefix0_words():
    out = []
    # Tick 0 is not an A tick.  The shortened tid0 contains positions 2 and 3.
    for p2 in ((0, 0), (0, 1), (1, 0)):
        for p3 in ((0, 0), (0, 1), (1, 0)):
            k32, k22 = p2
            k33, k23 = p3
            f0, f1, c2, c3 = 0, k32, k22, k23
            x2 = k33 & f1
            c2 ^= x2
            f0 ^= k33 ^ x2
            f1 ^= k33 ^ x2
            out.append((f0, f1, c2, c3))
    assert len(out) == len(set(out)) == 9
    return out


p0 = prefix0_words()
n = 9 * 26
mask = (1 << n) - 1
wires[:] = [0] * 23
for bit in range(4):
    wires[bit] = sum(1 << row for row in range(n) if p0[row // 26][bit])
for bit in range(5):
    wires[4 + bit] = sum(1 << row for row in range(n) if words[row % 26][bit])
initial = wires[:9].copy()
first_hit = len(hits)
run("prefix_commit", parse_ops("PREFIX_COMMIT"))
assert wires[3] == 0
run("prefix_pop", parse_ops("PREFIX_POP"))
assert wires[:9] == initial
assert all(v == 0 for v in wires[9:])
prefix_hits = hits[first_hit:]
print(f"prefix states={n} exact commit/pop roundtrip=yes")
print(f"constant/affine CCX candidates={len(prefix_hits)}")
for hit in prefix_hits:
    print(hit)
