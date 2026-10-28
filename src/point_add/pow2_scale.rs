//! Chunked modular scaling by powers of two for secp256k1.
//!
//! `mod_halve_by_k` computes `y := y * 2^-k mod p` in place with one
//! Montgomery-style reduction step instead of k single halvings. With
//! p = 2^256 - c, c = 2^32 + 977, and k <= 22:
//!
//!   g = (y mod 2^k) * 977^-1 mod 2^k        (so y - g*c = 0 mod 2^k)
//!   y - g*c = lo*2^k - F*2^256               (F = borrow bit)
//!   result  = (g - F)*2^(256-k) + lo
//!
//! F is uncomputed from the result as F = [lo*2^k + h*c >= p] with
//! h = g - F, i.e. the carry-out of lo*2^k + h*c + c.
//!
//! Cost is ~1.1k Toffoli per 22-bit chunk versus ~5.6k for 22 single
//! halvings. `mod_double_by_k` is the exact mirror.

use alloy_primitives::U256;

use super::{
    add_nbit_const_fast, add_nbit_qq_fast, cmp_lt_into_fast, mod_double_inplace_fast,
    mod_halve_inplace_fast, sub_nbit_const_fast, sub_nbit_qq_fast, QubitId, B,
};

pub(super) const MAX_CHUNK: usize = 22;
/// Below this many steps the single-bit doubling/halving is cheaper.
const MIN_CHUNK: usize = 4;
const C977: u64 = 977;

fn inv_977_mod_2k(k: usize) -> u64 {
    // Newton iteration for the inverse of an odd number mod 2^64.
    let mut inv: u64 = C977;
    for _ in 0..6 {
        inv = inv.wrapping_mul(2u64.wrapping_sub(C977.wrapping_mul(inv)));
    }
    debug_assert_eq!(C977.wrapping_mul(inv), 1);
    inv & ((1u64 << k) - 1)
}

/// Non-adjacent form digits (position, negative?) of `a`.
fn naf_digits(mut a: u64) -> Vec<(usize, bool)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while a != 0 {
        if a & 1 == 1 {
            let neg = a & 3 == 3;
            out.push((pos, neg));
            a = if neg { a.wrapping_add(1) } else { a - 1 };
        }
        a >>= 1;
        pos += 1;
    }
    out
}

/// `dst[shift..] (+/-)= src << shift  mod 2^dst.len()`, padding `src` with
/// zero ancillae as needed.
fn add_shifted(b: &mut B, src: &[QubitId], dst: &[QubitId], shift: usize, sub: bool) {
    if shift >= dst.len() {
        return;
    }
    let width = dst.len() - shift;
    let used = src.len().min(width);
    let pad = b.alloc_qubits(width - used);
    let mut a: Vec<QubitId> = src[..used].to_vec();
    a.extend_from_slice(&pad);
    if sub {
        sub_nbit_qq_fast(b, &a, &dst[shift..]);
    } else {
        add_nbit_qq_fast(b, &a, &dst[shift..]);
    }
    b.free_vec(&pad);
}

/// `dst := src * a mod 2^dst.len()` for odd `a` and zero `dst`
/// (`undo` = exact inverse: clears `dst` holding that product).
fn mul_const_mod2k(b: &mut B, src: &[QubitId], dst: &[QubitId], a: u64, undo: bool) {
    assert!(a & 1 == 1);
    // Bit 0 is a plain copy; the rest of `a` (even) goes through NAF adds.
    let digits = naf_digits(a - 1);
    let copy = src.len().min(dst.len());
    if !undo {
        for i in 0..copy {
            b.cx(src[i], dst[i]);
        }
        for &(pos, neg) in &digits {
            add_shifted(b, src, dst, pos, neg);
        }
    } else {
        for &(pos, neg) in digits.iter().rev() {
            add_shifted(b, src, dst, pos, !neg);
        }
        for i in 0..copy {
            b.cx(src[i], dst[i]);
        }
    }
}

/// `w := h * c` (c = 2^32 + 977) for zero `w`; `undo` clears it again.
/// Requires h.len() + 10 <= 32 so the two terms occupy disjoint bits.
fn times_c(b: &mut B, h: &[QubitId], w: &[QubitId], undo: bool) {
    let k = h.len();
    assert!(k + 10 <= 32 && w.len() >= 32 + k);
    let lowp = &w[..k + 10];
    if !undo {
        mul_const_mod2k(b, h, lowp, C977, false);
        for i in 0..k {
            b.cx(h[i], w[32 + i]);
        }
    } else {
        for i in 0..k {
            b.cx(h[i], w[32 + i]);
        }
        mul_const_mod2k(b, h, lowp, C977, true);
    }
}

fn reverse(b: &mut B, v: &[QubitId]) {
    let n = v.len();
    for i in 0..n / 2 {
        b.swap(v[i], v[n - 1 - i]);
    }
}

/// Rotate so that new v[i] = old v[(i + k) mod n].
fn rotate_down(b: &mut B, v: &[QubitId], k: usize) {
    let n = v.len();
    let k = k % n;
    if k == 0 {
        return;
    }
    reverse(b, &v[..k]);
    reverse(b, &v[k..]);
    reverse(b, v);
}

/// ext ^= carry-out of (r + h*c + c) over 2^256, where r = v with low k bits 0.
fn toggle_reduction_flag(b: &mut B, v: &[QubitId], g: &[QubitId], ext: QubitId) {
    let n = v.len();
    let k = g.len();
    let c = U256::from((1u64 << 32) + C977);
    let w = b.alloc_qubits(n);
    times_c(b, g, &w, false);
    add_nbit_const_fast(b, &w[..k + 34], c);
    for &q in v {
        b.x(q);
    }
    cmp_lt_into_fast(b, v, &w, ext);
    for &q in v {
        b.x(q);
    }
    sub_nbit_const_fast(b, &w[..k + 34], c);
    times_c(b, g, &w, true);
    b.free_vec(&w);
}

/// `v := v * 2^-k mod p` for v < p, 1 <= k <= MAX_CHUNK.
fn mod_halve_by_k(b: &mut B, v: &[QubitId], k: usize) {
    let n = v.len();
    assert!(n == 256 && (1..=MAX_CHUNK).contains(&k));
    let ext = b.alloc_qubit();
    let mut v_ext = v.to_vec();
    v_ext.push(ext);

    // g = v_low * 977^-1 mod 2^k.
    let g = b.alloc_qubits(k);
    mul_const_mod2k(b, &v[..k], &g, inv_977_mod_2k(k), false);

    // v_ext -= g*c  ->  F*2^256 + lo*2^k (low k bits now zero).
    let w = b.alloc_qubits(n + 1);
    times_c(b, &g, &w, false);
    sub_nbit_qq_fast(b, &w, &v_ext);
    times_c(b, &g, &w, true);
    b.free_vec(&w);

    // g -= F  ->  h.
    let d = b.alloc_qubits(k);
    b.cx(ext, d[0]);
    sub_nbit_qq_fast(b, &d, &g);
    b.cx(ext, d[0]);
    b.free_vec(&d);

    // F = carry(lo*2^k + h*c + c); clear it.
    toggle_reduction_flag(b, v, &g, ext);
    b.free(ext);

    // Move h into the (zero) low bits and rotate it to the top.
    for i in 0..k {
        b.swap(g[i], v[i]);
    }
    b.free_vec(&g);
    rotate_down(b, v, k);
}

/// `v := v * 2^k mod p` for v < p, 1 <= k <= MAX_CHUNK. Exact mirror of
/// `mod_halve_by_k`.
fn mod_double_by_k(b: &mut B, v: &[QubitId], k: usize) {
    let n = v.len();
    assert!(n == 256 && (1..=MAX_CHUNK).contains(&k));
    rotate_down(b, v, n - k);
    let g = b.alloc_qubits(k);
    for i in 0..k {
        b.swap(g[i], v[i]);
    }
    let ext = b.alloc_qubit();
    let mut v_ext = v.to_vec();
    v_ext.push(ext);

    toggle_reduction_flag(b, v, &g, ext);

    // h += F  ->  g.
    let d = b.alloc_qubits(k);
    b.cx(ext, d[0]);
    add_nbit_qq_fast(b, &d, &g);
    b.cx(ext, d[0]);
    b.free_vec(&d);

    let w = b.alloc_qubits(n + 1);
    times_c(b, &g, &w, false);
    add_nbit_qq_fast(b, &w, &v_ext);
    times_c(b, &g, &w, true);
    b.free_vec(&w);
    b.free(ext);

    mul_const_mod2k(b, &v[..k], &g, inv_977_mod_2k(k), true);
    b.free_vec(&g);
}

fn chunks(count: usize) -> (Vec<usize>, usize) {
    let full = count / MAX_CHUNK;
    let rem = count % MAX_CHUNK;
    let mut ks = vec![MAX_CHUNK; full];
    if rem >= MIN_CHUNK {
        ks.push(rem);
        (ks, 0)
    } else {
        (ks, rem)
    }
}

/// `v := v * 2^-count mod p`.
pub(super) fn mod_halve_pow2(b: &mut B, v: &[QubitId], p: U256, count: usize) {
    let (ks, singles) = chunks(count);
    for k in ks {
        mod_halve_by_k(b, v, k);
    }
    for _ in 0..singles {
        mod_halve_inplace_fast(b, v, p);
    }
}

/// `v := v * 2^count mod p`.
pub(super) fn mod_double_pow2(b: &mut B, v: &[QubitId], p: U256, count: usize) {
    let (ks, singles) = chunks(count);
    for _ in 0..singles {
        mod_double_inplace_fast(b, v, p);
    }
    for k in ks.into_iter().rev() {
        mod_double_by_k(b, v, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::QubitOrBit;
    use crate::sim::Simulator;
    use sha3::digest::{ExtendableOutput, Update};

    fn p() -> U256 {
        super::super::SECP256K1_P
    }

    fn pow2(k: usize) -> U256 {
        U256::from(1u64) << k
    }

    fn mulmod(a: U256, b: U256) -> U256 {
        a.mul_mod(b, p())
    }

    fn run(op: &dyn Fn(&mut B, &[QubitId]), inputs: &[U256]) -> Vec<(U256, bool, bool)> {
        let mut b = B::new();
        let x = b.alloc_qubits(256);
        op(&mut b, &x);
        let ops = b.ops.clone();
        let total_qubits = b.next_qubit as usize;
        let num_bits = b.next_bit as usize;
        let reg: Vec<QubitOrBit> = x.iter().map(|&q| QubitOrBit::Qubit(q)).collect();
        let mut h = sha3::Shake256::default();
        h.update(b"pow2-test");
        let mut xof = h.finalize_xof();
        let mut sim = Simulator::new(total_qubits, num_bits, &mut xof);
        let mut out = Vec::new();
        for chunk in inputs.chunks(64) {
            sim.clear_for_shot();
            for (s, &v) in chunk.iter().enumerate() {
                sim.set_register(&reg, v, s);
            }
            sim.apply_iter(ops.iter());
            for s in 0..chunk.len() {
                let got = sim.get_register(&reg, s);
                let phase_bad = (sim.phase >> s) & 1 == 1;
                let mut dirty = false;
                for q in 0..total_qubits {
                    if x.iter().any(|xq| xq.0 as usize == q) {
                        continue;
                    }
                    if (sim.qubits[q] >> s) & 1 == 1 {
                        dirty = true;
                    }
                }
                out.push((got, phase_bad, dirty));
            }
        }
        out
    }

    fn inputs() -> Vec<U256> {
        let mut v = vec![
            U256::ZERO,
            U256::from(1u64),
            U256::from(2u64),
            p() - U256::from(1u64),
            p() - U256::from(2u64),
            p() >> 1,
            (p() >> 1) + U256::from(1u64),
        ];
        // Inputs near the reduction boundary for every chunk size.
        for k in 1..=MAX_CHUNK {
            let top = p() >> (256 - k);
            for dh in 0..3u64 {
                for dl in 0..3u64 {
                    let h = top.wrapping_sub(U256::from(dh));
                    let lo = (pow2(256 - k) - U256::from(1u64)).wrapping_sub(U256::from(dl));
                    let x = (h << (256 - k)) | lo;
                    if x < p() {
                        v.push(x);
                    }
                }
            }
        }
        let mut s: u64 = 0x1234_5678_9abc_def1;
        while v.len() < 64 * 12 {
            let mut limbs = [0u64; 4];
            for l in &mut limbs {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                *l = s;
            }
            let x = U256::from_limbs(limbs);
            if x < p() {
                v.push(x);
            }
        }
        v
    }

    fn check(name: &str, count: usize, double: bool) {
        let ins = inputs();
        let res = if double {
            run(&|b, x| mod_double_pow2(b, x, p(), count), &ins)
        } else {
            run(&|b, x| mod_halve_pow2(b, x, p(), count), &ins)
        };
        let f = if double {
            let mut r = U256::from(1u64);
            for _ in 0..count {
                r = mulmod(r, U256::from(2u64));
            }
            r
        } else {
            // 2^-count = ((p+1)/2)^count
            let half = (p() + U256::from(1u64)) >> 1;
            let mut r = U256::from(1u64);
            for _ in 0..count {
                r = mulmod(r, half);
            }
            r
        };
        let mut bad = 0;
        for (i, (&x, &(got, ph, dirty))) in ins.iter().zip(res.iter()).enumerate() {
            let exp = mulmod(x, f);
            if got != exp || ph || dirty {
                bad += 1;
                if bad < 5 {
                    eprintln!("{name} input#{i} x={x:#x} got={got:#x} exp={exp:#x} phase={ph} dirty={dirty}");
                }
            }
        }
        assert_eq!(bad, 0, "{name}: {bad} bad shots");
    }

    #[test]
    fn halve_chunks() {
        for &c in &[4usize, 8, 21, 22, 404] {
            check("halve", c, false);
        }
    }

    #[test]
    fn double_chunks() {
        for &c in &[4usize, 8, 21, 22, 400] {
            check("double", c, true);
        }
    }
}
