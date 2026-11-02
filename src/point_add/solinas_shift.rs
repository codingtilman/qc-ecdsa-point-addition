//! Multi-bit Montgomery-style modular halving / doubling for secp256k1.
//!
//! p = 2^256 - c with c = 2^32 + 977. To compute r = v * 2^-k mod p
//! (0 <= v < p, 1 <= k <= 22):
//!   L = v mod 2^k
//!   M = L * c^-1 mod 2^k          (so v + M*p ≡ 0 mod 2^k)
//!   r = (v + M*p) / 2^k = (v - M*c) / 2^k + M * 2^(256-k)
//! and r < p exactly whenever v < p. The circuit computes M out of place
//! from L, subtracts T = M*c (a (k+33)-bit value: M*977 in the low 32 bits,
//! M at bit 32) from v, rotates the now-zero low k bits to the top, then
//! swaps M into them. The 256-bit subtraction drops its borrow, which is
//! only wrong when v < M*c (probability ~2^(k+33-256), same class as the
//! single-bit `mod_halve_inplace_fast` corner case).
//!
//! Cost per k-bit round ≈ 255 (big sub) + ~6(k+10) (T compute/uncompute)
//! + ~k^2/6 (M) Toffoli, versus 255*k for k single halvings.
//!
//! Doubling (r = v * 2^k mod p) is the exact reverse sequence.

use super::*;

const C_LOW: u64 = 977;

fn inv_mod_2k(k: usize) -> u64 {
    // Newton iteration for 977^-1 mod 2^64, then mask.
    let mut x: u64 = 1;
    for _ in 0..7 {
        x = x.wrapping_mul(2u64.wrapping_sub(C_LOW.wrapping_mul(x)));
    }
    debug_assert_eq!(C_LOW.wrapping_mul(x), 1);
    if k >= 64 {
        x
    } else {
        x & ((1u64 << k) - 1)
    }
}

/// Non-adjacent form digits (position, sign) of x.
fn naf(x: u64) -> Vec<(usize, i8)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    // Use i128 to avoid overflow on the +1 carry.
    let mut xx = x as i128;
    while xx != 0 {
        if xx & 1 == 1 {
            let z: i128 = 2 - (xx & 3);
            xx -= z;
            out.push((i, z as i8));
        }
        xx >>= 1;
        i += 1;
    }
    out
}

/// acc (window) += / -= src (truncated or zero-padded to the window width),
/// modulo 2^width.
fn add_window(b: &mut B, acc: &[QubitId], src: &[QubitId], sub: bool) {
    let w = acc.len();
    if w == 0 {
        return;
    }
    let mut opnd: Vec<QubitId> = src.iter().take(w).cloned().collect();
    let pad = if opnd.len() < w {
        let p = b.alloc_qubits(w - opnd.len());
        opnd.extend_from_slice(&p);
        p
    } else {
        Vec::new()
    };
    if sub {
        sub_nbit_qq_fast(b, &opnd, acc);
    } else {
        add_nbit_qq_fast(b, &opnd, acc);
    }
    b.free_vec(&pad);
}

/// m ^= / -= (l * d mod 2^k) where `digits` is the NAF of d (d odd).
/// Forward requires m = 0 on entry; uncompute requires m = l*d on entry.
fn m_from_l(b: &mut B, l: &[QubitId], m: &[QubitId], digits: &[(usize, i8)], uncompute: bool) {
    let k = m.len();
    assert_eq!(digits[0], (0, 1));
    let apply = |b: &mut B, (j, s): (usize, i8), undo: bool| {
        if j >= k {
            return;
        }
        if j == 0 && s == 1 {
            for i in 0..k {
                b.cx(l[i], m[i]);
            }
        } else {
            let neg = (s < 0) != undo;
            add_window(b, &m[j..k], l, neg);
        }
    };
    if uncompute {
        for &dg in digits.iter().rev() {
            apply(b, dg, true);
        }
    } else {
        for &dg in digits.iter() {
            apply(b, dg, false);
        }
    }
}

/// a (k+10 bits) ^= / -= m * 977. 977 = 2^10 - 2^6 + 2^4 + 1.
fn a_from_m(b: &mut B, m: &[QubitId], a: &[QubitId], uncompute: bool) {
    let k = m.len();
    let w = a.len();
    let terms: [(usize, bool); 3] = [(4, false), (6, true), (10, false)];
    if !uncompute {
        for i in 0..k {
            b.cx(m[i], a[i]);
        }
        for &(j, neg) in terms.iter() {
            add_window(b, &a[j..w], m, neg);
        }
    } else {
        for &(j, neg) in terms.iter().rev() {
            add_window(b, &a[j..w], m, !neg);
        }
        for i in 0..k {
            b.cx(m[i], a[i]);
        }
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// new v[i] = old v[(i + k) mod n], using swaps along permutation cycles.
pub(super) fn rotate_right_by(b: &mut B, v: &[QubitId], k: usize) {
    let n = v.len();
    let k = k % n;
    if k == 0 {
        return;
    }
    let g = gcd(n, k);
    for s in 0..g {
        let mut cur = s;
        loop {
            let nxt = (cur + k) % n;
            if nxt == s {
                break;
            }
            b.swap(v[cur], v[nxt]);
            cur = nxt;
        }
    }
}

/// v -= / += T where T = a (low k+10 bits) + m * 2^32, full 256-bit width.
fn apply_t(b: &mut B, v: &[QubitId], a: &[QubitId], m: &[QubitId], sub: bool, lowq: bool) {
    let n = v.len();
    let k = m.len();
    let mut opnd: Vec<QubitId> = a.to_vec();
    let mut pads: Vec<QubitId> = Vec::new();
    let gap = b.alloc_qubits(32 - a.len());
    opnd.extend_from_slice(&gap);
    pads.extend_from_slice(&gap);
    opnd.extend_from_slice(m);
    let top = b.alloc_qubits(n - 32 - k); // n = 256, or 257 with the borrow bit
    opnd.extend_from_slice(&top);
    pads.extend_from_slice(&top);
    match (sub, lowq) {
        (true, false) => sub_nbit_qq_fast(b, &opnd, v),
        (false, false) => add_nbit_qq_fast(b, &opnd, v),
        (true, true) => sub_nbit_qq(b, &opnd, v),
        (false, true) => add_nbit_qq(b, &opnd, v),
    }
    b.free_vec(&pads);
}

/// v := v * 2^-k mod p (secp256k1), 1 <= k <= 22.
pub(super) fn mod_halve_by_k(b: &mut B, v: &[QubitId], k: usize) {
    halve_by_k_impl(b, v, k, false)
}

/// Like `mod_halve_by_k` but the 256-bit subtraction uses the carry-free
/// Cuccaro (no 255-qubit carry register) for peak-sensitive call sites.
pub(super) fn mod_halve_by_k_lowq(b: &mut B, v: &[QubitId], k: usize) {
    halve_by_k_impl(b, v, k, true)
}

fn halve_by_k_impl(b: &mut B, v: &[QubitId], k: usize, lowq: bool) {
    let n = v.len();
    assert_eq!(n, 256);
    assert!(k >= 1 && k <= 22);
    let digits = naf(inv_mod_2k(k));
    let m = b.alloc_qubits(k);
    m_from_l(b, &v[0..k], &m, &digits, false);
    let a = b.alloc_qubits(k + 10);
    a_from_m(b, &m, &a, false);
    // Exact: keep the borrow e = [v < M*c] in a 257th bit.
    let e = b.alloc_qubit();
    let mut ve = v.to_vec();
    ve.push(e);
    apply_t(b, &ve, &a, &m, true, lowq);
    a_from_m(b, &m, &a, true);
    b.free_vec(&a);
    // Low k bits of v are now zero.
    rotate_right_by(b, v, k);
    for i in 0..k {
        b.swap(m[i], v[n - k + i]);
    }
    b.free_vec(&m);
    // Z = v + M p has top field M - e: fix it, then clear e from the result.
    sub_bit_from_field(b, &v[n - k..n], e);
    borrow_test(b, v, k, e, lowq);
    b.free(e);
}

/// field -= bit (mod 2^len).
fn sub_bit_from_field(b: &mut B, field: &[QubitId], bit_q: QubitId) {
    let pads = b.alloc_qubits(field.len() - 1);
    let mut op = vec![bit_q];
    op.extend_from_slice(&pads);
    sub_nbit_qq_fast(b, &op, field);
    b.free_vec(&pads);
}
fn add_bit_to_field(b: &mut B, field: &[QubitId], bit_q: QubitId) {
    let pads = b.alloc_qubits(field.len() - 1);
    let mut op = vec![bit_q];
    op.extend_from_slice(&pads);
    add_nbit_qq_fast(b, &op, field);
    b.free_vec(&pads);
}

/// e ^= [X0 >= p] for r = v (r < p), with F = r >> (256-k), r_low = r mod
/// 2^(256-k), X0 = r_low*2^k + F*c. This equals the borrow of the halving
/// round that produced r: v_in = X0 - e*p with 0 <= v_in < p.
/// [X0 >= p] = carry-out of r_low + G (over 256-k bits), G = floor((F+1)c/2^k).
/// Chain AND helpers (len >= 2). `and_compute` leaves chain ancillas holding
/// prefix ANDs; `and_uncompute` clears them by measurement.
fn and_compute(b: &mut B, x: &[QubitId]) -> Vec<QubitId> {
    let ch = b.alloc_qubits(x.len() - 1);
    b.ccx(x[0], x[1], ch[0]);
    for j in 2..x.len() {
        b.ccx(ch[j - 2], x[j], ch[j - 1]);
    }
    ch
}
fn and_uncompute(b: &mut B, x: &[QubitId], ch: Vec<QubitId>) {
    for j in (2..x.len()).rev() {
        let mm = b.alloc_bit();
        b.hmr(ch[j - 1], mm);
        b.cz_if(ch[j - 2], x[j], mm);
    }
    let mm = b.alloc_bit();
    b.hmr(ch[0], mm);
    b.cz_if(x[0], x[1], mm);
    b.free_vec(&ch);
}
/// t ^= AND(x), one ancilla per control.
fn and_into(b: &mut B, x: &[QubitId], t: QubitId) {
    let ch = and_compute(b, x);
    b.cx(*ch.last().unwrap(), t);
    and_uncompute(b, x, ch);
}
/// t ^= AND(x) with ~2*sqrt(len) ancillas: AND each block into a kept qubit
/// (chain cleared by measurement), AND the block results into t, then clear
/// each kept qubit by measurement with its phase fix recomputed from the
/// block (about 2 Toffoli per control instead of 1).
fn and_into_blocked(b: &mut B, x: &[QubitId], t: QubitId) {
    const BLK: usize = 16;
    let blocks: Vec<&[QubitId]> = x.chunks(BLK).collect();
    let mut keep: Vec<QubitId> = Vec::new();
    for blk in &blocks {
        if blk.len() == 1 {
            keep.push(blk[0]);
            continue;
        }
        let q = b.alloc_qubit();
        and_into(b, blk, q);
        keep.push(q);
    }
    and_into(b, &keep, t);
    for (blk, &q) in blocks.iter().zip(keep.iter()) {
        if blk.len() == 1 {
            continue;
        }
        let mm = b.alloc_bit();
        b.hmr(q, mm);
        // phase fix: Z on AND(blk), conditioned on mm
        if blk.len() == 2 {
            b.cz_if(blk[0], blk[1], mm);
        } else {
            let pre = &blk[..blk.len() - 1];
            let ch = and_compute(b, pre);
            b.cz_if(*ch.last().unwrap(), blk[blk.len() - 1], mm);
            and_uncompute(b, pre, ch);
        }
        b.free(q);
    }
}

fn borrow_test(b: &mut B, r: &[QubitId], k: usize, e: QubitId, lowq: bool) {
    let n = r.len();
    let w = n - k;
    let f = &r[w..n];
    // F1 = F + 1 (k+1 bits, no overflow)
    let f1 = b.alloc_qubits(k + 1);
    for i in 0..k {
        b.cx(f[i], f1[i]);
    }
    sparse_const::add_const(b, &f1, U256::from(1u64), None, false);
    // A1 = F1 * 977 < 2^(k+10) <= 2^32
    let a1 = b.alloc_qubits(k + 10);
    a_from_m(b, &f1, &a1, false);
    // G (33 bits) = (A1 >> k) | gap zeros | F1 at bit 32-k
    let gap = b.alloc_qubits(32 - k - 10);
    let mut g: Vec<QubitId> = a1[k..k + 10].to_vec();
    g.extend_from_slice(&gap);
    g.extend_from_slice(&f1);
    let gw = g.len();
    debug_assert_eq!(gw, 33);
    // carry of r_low[0..gw] + G  ==  (~r_low[0..gw] < G)
    for i in 0..gw {
        b.x(r[i]);
    }
    let flag = b.alloc_qubit();
    let hi: Vec<QubitId> = r[gw..w].to_vec();
    with_lt(b, &r[0..gw], &g, flag, |b| {
        // carry propagates through the upper bits iff they are all ones
        let mut ctl = vec![flag];
        ctl.extend_from_slice(&hi);
        if lowq {
            and_into_blocked(b, &ctl, e);
        } else {
            and_into(b, &ctl, e);
        }
    });
    b.free(flag);
    for i in 0..gw {
        b.x(r[i]);
    }
    b.free_vec(&gap);
    a_from_m(b, &f1, &a1, true);
    b.free_vec(&a1);
    sparse_const::add_const(b, &f1, U256::from(1u64), None, true);
    for i in 0..k {
        b.cx(f[i], f1[i]);
    }
    b.free_vec(&f1);
}

/// v := v * 2^k mod p (secp256k1), 1 <= k <= 22. Exact reverse of
/// `mod_halve_by_k`.
pub(super) fn mod_double_by_k(b: &mut B, v: &[QubitId], k: usize) {
    double_by_k_impl(b, v, k, false)
}

pub(super) fn mod_double_by_k_lowq(b: &mut B, v: &[QubitId], k: usize) {
    double_by_k_impl(b, v, k, true)
}

fn double_by_k_impl(b: &mut B, v: &[QubitId], k: usize, lowq: bool) {
    let n = v.len();
    assert_eq!(n, 256);
    assert!(k >= 1 && k <= 22);
    let digits = naf(inv_mod_2k(k));
    // Exact reverse of halve_by_k_impl.
    let e = b.alloc_qubit();
    borrow_test(b, v, k, e, lowq);
    add_bit_to_field(b, &v[n - k..n], e);
    let m = b.alloc_qubits(k);
    for i in 0..k {
        b.swap(m[i], v[n - k + i]);
    }
    rotate_right_by(b, v, n - k);
    let a = b.alloc_qubits(k + 10);
    a_from_m(b, &m, &a, false);
    let mut ve = v.to_vec();
    ve.push(e);
    apply_t(b, &ve, &a, &m, false, lowq);
    a_from_m(b, &m, &a, true);
    b.free_vec(&a);
    b.free(e);
    m_from_l(b, &v[0..k], &m, &digits, true);
    b.free_vec(&m);
}

/// v := v * 2^-total mod p.
pub(super) fn mod_halve_many(b: &mut B, v: &[QubitId], total: usize) {
    let mut rem = total;
    while rem > 0 {
        let k = rem.min(22);
        mod_halve_by_k(b, v, k);
        rem -= k;
    }
}

/// v := v * 2^total mod p.
pub(super) fn mod_double_many(b: &mut B, v: &[QubitId], total: usize) {
    let mut rem = total;
    while rem > 0 {
        let k = rem.min(22);
        mod_double_by_k(b, v, k);
        rem -= k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> U256 {
        SECP256K1_P
    }

    fn run(k: usize, halve: bool, lowq: bool, vals: &[U256]) -> Vec<U256> {
        let mut b = B::new();
        let v = b.alloc_qubits(256);
        let ops_start = b.ops.len();
        match (halve, lowq) {
            (true, false) => mod_halve_by_k(&mut b, &v, k),
            (true, true) => mod_halve_by_k_lowq(&mut b, &v, k),
            (false, false) => mod_double_by_k(&mut b, &v, k),
            (false, true) => mod_double_by_k_lowq(&mut b, &v, k),
        }
        let mut h = Shake256::default();
        h.update(b"selftest");
        let mut xof = h.finalize_xof();
        let mut sim = Simulator::new(b.next_qubit as usize, b.next_bit as usize, &mut xof);
        for (lane, val) in vals.iter().enumerate() {
            for i in 0..256 {
                if bit(*val, i) {
                    sim.qubits[v[i].0 as usize] |= 1u64 << lane;
                }
            }
        }
        sim.apply_iter(b.ops[ops_start..].iter());
        assert_eq!(sim.phase, 0, "phase garbage k={k} halve={halve} lowq={lowq}");
        let vset: std::collections::HashSet<u64> = v.iter().map(|q| q.0).collect();
        for q in 0..b.next_qubit {
            if !vset.contains(&q) {
                assert_eq!(sim.qubits[q as usize], 0, "dirty ancilla {q}");
            }
        }
        (0..vals.len())
            .map(|lane| {
                let mut x = U256::ZERO;
                for i in 0..256 {
                    if (sim.qubits[v[i].0 as usize] >> lane) & 1 == 1 {
                        x |= U256::from(1u64) << i;
                    }
                }
                x
            })
            .collect()
    }

    fn inputs() -> Vec<U256> {
        let p = p();
        let mut out = Vec::new();
        let mut h = Shake256::default();
        h.update(b"inputs");
        let mut rd = h.finalize_xof();
        // adversarial: tiny values (the old borrow-dropping failure set),
        // values just below p, and random values.
        for t in [0u64, 1, 2, 3, 977, (1u64 << 33) + 5, (1u64 << 54) - 1, 1u64 << 54, u64::MAX] {
            out.push(U256::from(t));
        }
        for t in 1u64..=6 {
            out.push(p - U256::from(t));
        }
        out.push(p - (U256::from(1u64) << 200));
        while out.len() < 64 {
            let mut buf = [0u8; 32];
            rd.read(&mut buf);
            let x = U256::from_le_bytes(buf) % p;
            // also small random values that trigger the borrow
            if out.len() % 4 == 0 {
                out.push(x >> 200);
            } else {
                out.push(x);
            }
        }
        out
    }

    #[test]
    fn exact_k_bit_rounds() {
        let p = p();
        let inv2: U256 = (p + U256::from(1u64)) >> 1usize;
        let vals = inputs();
        for &k in &[1usize, 4, 8, 10, 21, 22] {
            for &lowq in &[false, true] {
                let scale_dn = inv2.pow_mod(U256::from(k as u64), p);
                let scale_up = U256::from(2u64).pow_mod(U256::from(k as u64), p);
                let got = run(k, true, lowq, &vals);
                for (x, y) in vals.iter().zip(got.iter()) {
                    assert_eq!(*y, x.mul_mod(scale_dn, p), "halve k={k} lowq={lowq} x={x:#x}");
                }
                let got = run(k, false, lowq, &vals);
                for (x, y) in vals.iter().zip(got.iter()) {
                    assert_eq!(*y, x.mul_mod(scale_up, p), "double k={k} lowq={lowq} x={x:#x}");
                }
            }
        }
    }
}
