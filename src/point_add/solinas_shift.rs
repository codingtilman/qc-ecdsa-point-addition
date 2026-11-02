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
fn rotate_right_by(b: &mut B, v: &[QubitId], k: usize) {
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
    let top = b.alloc_qubits(n - 32 - k);
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
    apply_t(b, v, &a, &m, true, lowq);
    a_from_m(b, &m, &a, true);
    b.free_vec(&a);
    // Low k bits of v are now zero.
    rotate_right_by(b, v, k);
    for i in 0..k {
        b.swap(m[i], v[n - k + i]);
    }
    b.free_vec(&m);
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
    let m = b.alloc_qubits(k);
    for i in 0..k {
        b.swap(m[i], v[n - k + i]);
    }
    rotate_right_by(b, v, n - k);
    let a = b.alloc_qubits(k + 10);
    a_from_m(b, &m, &a, false);
    apply_t(b, v, &a, &m, false, lowq);
    a_from_m(b, &m, &a, true);
    b.free_vec(&a);
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
