//! Half-cost exact squaring via controlled add-subtract rows.
//!
//! Row i (0 <= i <= n-2), T_i = x[i+1..n] (L = n-i-1 bits), ctrl = x[i]:
//!   W[2i+1 .. n+i+1) += x_i ? T_i : (2^L - T_i)
//! (`controlled_add_subtract_fast`, ~L Toffoli). Every row adds a value in
//! [0, 2^L], and processing rows low-to-high keeps W < 2^(n+i) before row i,
//! so the (L+1)-bit window never carries out. Afterwards
//!   W = 2*S - C + Off,  S = sum_{i<j} x_i x_j 2^(i+j),
//!   C = sum_{i<j} x_j 2^(i+j) = spread(x) - x,  spread(x) = sum x_j 4^j,
//!   Off = 2^n (2^(n-1) - 1 - x_low),  x_low = x mod 2^(n-1),
//! so x^2 = spread(x) + 2S = W + 2*spread(x) - x + 2^n (x_low + 1) - 2^(2n-1)
//! (mod 2^(2n)). Total ~ n^2/2 + 5n Toffoli vs ~n^2 for the AND-row version.

use super::*;

fn corrections(b: &mut B, x: &[QubitId], w: &[QubitId], inverse: bool) {
    let n = x.len();
    // (a) W += 2*spread(x)
    let zeros = b.alloc_qubits(n);
    let mut spread: Vec<QubitId> = Vec::with_capacity(2 * n);
    for j in 0..n {
        spread.push(zeros[j]);
        spread.push(x[j]);
    }
    // (b) W -= x  (x zero-padded to 2n)
    let mut x_pad: Vec<QubitId> = x.to_vec();
    x_pad.extend_from_slice(&zeros);
    // (c) W[n..2n] += x_low + 1, then W[2n-1] ^= 1
    let mut xl: Vec<QubitId> = x[..n - 1].to_vec();
    xl.push(zeros[0]);
    let top: Vec<QubitId> = w[n..2 * n].to_vec();
    if !inverse {
        add_nbit_qq_fast(b, &spread, w);
        sub_nbit_qq_fast(b, &x_pad, w);
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_add_fast(b, &xl, &top, c_in);
        b.x(c_in);
        b.free(c_in);
        b.x(w[2 * n - 1]);
    } else {
        b.x(w[2 * n - 1]);
        let c_in = b.alloc_qubit();
        b.x(c_in);
        cuccaro_sub_fast(b, &xl, &top, c_in);
        b.x(c_in);
        b.free(c_in);
        add_nbit_qq_fast(b, &x_pad, w);
        sub_nbit_qq_fast(b, &spread, w);
    }
    b.free_vec(&zeros);
}

/// tmp_ext (2n qubits, zero) := x^2.
pub(super) fn square_into_zero(b: &mut B, x: &[QubitId], w: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(w.len(), 2 * n);
    for i in 0..n - 1 {
        let l = n - i - 1;
        let t: Vec<QubitId> = x[i + 1..n].to_vec();
        let acc: Vec<QubitId> = w[2 * i + 1..2 * i + 1 + l + 1].to_vec();
        controlled_add_subtract_fast(b, &t, &acc, x[i]);
    }
    corrections(b, x, w, false);
}

/// Exact inverse of `square_into_zero` (tmp_ext := 0).
pub(super) fn square_into_zero_inverse(b: &mut B, x: &[QubitId], w: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(w.len(), 2 * n);
    corrections(b, x, w, true);
    for i in (0..n - 1).rev() {
        let l = n - i - 1;
        let t: Vec<QubitId> = x[i + 1..n].to_vec();
        let acc: Vec<QubitId> = w[2 * i + 1..2 * i + 1 + l + 1].to_vec();
        controlled_add_subtract_fast_inverse(b, &t, &acc, x[i]);
    }
}
