#!/usr/bin/env python3
"""Exhaustive multiplicative-complexity search for the A2 codec reads.

The five stored bits are (f0,f1,c1,c2,c3).  Legal logical states have
wt(k3_1,k3_2,k3_3) <= 2 and k2_i = 0 whenever k3_i = 1.  The encoder below
is a classical transcription of a2_code_commit for a complete three-tick
block.  Search networks have two fresh AND wires:

    a = affine(input) & affine(input)
    b = affine(input,a) & affine(input,a)

and arbitrary affine output maps.  Truth functions are represented as
bitsets over the 26 legal encoded states.
"""

from itertools import combinations


def encode(k3, k2):
    f0 = k3[0]
    f1 = k3[1]
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


def decode(x):
    f0, f1, c1, c2, c3 = x
    k31 = f0 & (1 ^ (f1 & (1 ^ c3)))
    k32 = (f0 ^ f1) & (f1 ^ c1 ^ (f1 & c1))
    k33 = f1 & (f0 ^ c2 ^ (f0 & c2))
    k3 = (k31, k32, k33)
    k2 = tuple((1 ^ k3[i]) & (c1, c2, c3)[i] for i in range(3))
    return k3, k2


logical = []
for k3n in range(8):
    k3 = tuple((k3n >> i) & 1 for i in range(3))
    if sum(k3) > 2:
        continue
    free = [i for i in range(3) if not k3[i]]
    for bits in range(1 << len(free)):
        k2 = [0, 0, 0]
        for j, i in enumerate(free):
            k2[i] = (bits >> j) & 1
        logical.append((k3, tuple(k2)))

states = [encode(*s) for s in logical]
assert len(logical) == 26
assert len(set(states)) == 26
for state, want in zip(states, logical):
    assert decode(state) == want, (state, decode(state), want)

mask = (1 << len(states)) - 1
one = mask
xs = []
for j in range(5):
    xs.append(sum(((state[j] & 1) << i) for i, state in enumerate(states)))


def affine_truth(coeff, basis):
    out = 0
    for i, v in enumerate(basis):
        if (coeff >> i) & 1:
            out ^= v
    return out


def gf2_basis(vectors):
    piv = {}
    for value, recipe in vectors:
        v = value
        r = recipe
        while v:
            p = v.bit_length() - 1
            if p not in piv:
                piv[p] = (v, r)
                break
            v ^= piv[p][0]
            r ^= piv[p][1]
    return piv


def reduce_with(value, piv):
    v = value
    recipe = 0
    while v:
        p = v.bit_length() - 1
        if p not in piv:
            return v, recipe
        v ^= piv[p][0]
        recipe ^= piv[p][1]
    return 0, recipe


def fmt_aff(coeff, names):
    terms = [names[i] for i in range(len(names)) if (coeff >> i) & 1]
    return " ^ ".join(terms) if terms else "0"


base_inputs = [one] + xs
input_names = ["1", "f0", "f1", "c1", "c2", "c3"]

# Unique first-AND truth functions, retaining one factorization.
first_products = {}
forms0 = [(c, affine_truth(c, base_inputs)) for c in range(1, 1 << 6)]
for i, (ca, a) in enumerate(forms0):
    for cb, b in forms0[i:]:
        p = a & b
        first_products.setdefault(p, (ca, cb))

print(f"legal states={len(states)} unique first products={len(first_products)}")
print("encoded states:")
for logical_state, state in zip(logical, states):
    print(f"  {logical_state} -> {state}")

for pos in range(3):
    y3 = sum((logical[i][0][pos] << i) for i in range(26))
    y2 = sum((logical[i][1][pos] << i) for i in range(26))
    found = None

    for aval, (aa, ab) in first_products.items():
        basis = base_inputs + [aval]
        # Recipe bits correspond to [1,f0,f1,c1,c2,c3,a].
        piv = gf2_basis((v, 1 << i) for i, v in enumerate(basis))
        r3, rec3 = reduce_with(y3, piv)
        r2, rec2 = reduce_with(y2, piv)
        if r3 == 0 and r2 == 0:
            found = (aa, ab, 0, 0, rec3, rec2, aval, None)
            break
        if r3 and r2 and r3 != r2:
            continue
        target_residue = r3 or r2

        # Unique second products of affine forms in (inputs,a).
        second_products = {}
        forms1 = [(c, affine_truth(c, basis)) for c in range(1, 1 << 7)]
        for i, (ba, va) in enumerate(forms1):
            for bb, vb in forms1[i:]:
                product = va & vb
                residue, _ = reduce_with(product, piv)
                if residue == target_residue:
                    second_products[product] = (ba, bb)
                    break
            if second_products:
                break
        if not second_products:
            continue
        bval, (ba, bb) = next(iter(second_products.items()))
        rb, recb = reduce_with(bval, piv)
        assert rb == target_residue
        # y = base_recipe ^ b when its residue is nonzero.
        out3 = rec3 ^ ((1 << 7) if r3 else 0) ^ (recb if r3 else 0)
        out2 = rec2 ^ ((1 << 7) if r2 else 0) ^ (recb if r2 else 0)
        found = (aa, ab, ba, bb, out3, out2, aval, bval)
        break

    print(f"position {pos + 1}:")
    if found is None:
        print("  no network with <=2 ANDs")
        continue
    aa, ab, ba, bb, out3, out2, aval, bval = found
    print(f"  a = ({fmt_aff(aa, input_names)}) & ({fmt_aff(ab, input_names)})")
    if bval is not None:
        names1 = input_names + ["a"]
        print(f"  b = ({fmt_aff(ba, names1)}) & ({fmt_aff(bb, names1)})")
        out_names = input_names + ["a", "b"]
    else:
        out_names = input_names + ["a", "b"]
    print(f"  k3 = {fmt_aff(out3, out_names)}")
    print(f"  k2 = {fmt_aff(out2, out_names)}")

