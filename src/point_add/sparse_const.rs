//! Constant addition `v ± ctrl·c (mod 2^n)` with only carry ancillae.
//!
//! Gidney-style ripple carry: carry c_{i+1} = MAJ(a_i, v_i, c_i) is computed
//! into a fresh ancilla with one Toffoli per bit and uncomputed by
//! measurement. The operand a_i is classical:
//!   * c-bit 0:            c_{i+1} = v_i & c_i
//!   * c-bit 1, no ctrl:   c_{i+1} = v_i | c_i
//!   * c-bit 1, ctrl q:    c_{i+1} = c_i ^ ((q ^ c_i) & (v_i ^ c_i)), with q
//!                         toggled in place by c_i and restored (shared ctrl)
//! so no operand register is needed: ~n-1 ancillae and n-1 Toffoli, versus
//! n operand qubits + n-1 carries for the loaded-constant Cuccaro.
//! Subtraction uses v - a = ~(~v + a).

use super::*;

pub(super) fn add_const(b: &mut B, v: &[QubitId], c: U256, ctrl: Option<QubitId>, subtract: bool) {
    let n = v.len();
    if n == 0 {
        return;
    }
    if subtract {
        for &q in v {
            b.x(q);
        }
    }
    // carries[i] = carry into bit i (None = known zero).
    let mut carries: Vec<Option<QubitId>> = vec![None; n];
    // Forward: compute carries into bits 1..n-1.
    for i in 0..n - 1 {
        let ci = carries[i];
        let cb = bit(c, i);
        match (ci, cb) {
            (None, false) => {}
            (None, true) => {
                let q = b.alloc_qubit();
                match ctrl {
                    Some(t) => b.ccx(t, v[i], q),
                    None => b.cx(v[i], q),
                }
                carries[i + 1] = Some(q);
            }
            (Some(cq), false) => {
                let q = b.alloc_qubit();
                b.ccx(v[i], cq, q);
                carries[i + 1] = Some(q);
            }
            (Some(cq), true) => {
                let q = b.alloc_qubit();
                match ctrl {
                    Some(t) => {
                        b.cx(cq, t);
                        b.cx(cq, v[i]);
                        b.ccx(t, v[i], q);
                        b.cx(cq, q);
                        b.cx(cq, t);
                    }
                    None => or_step(b, v[i], cq, q),
                }
                carries[i + 1] = Some(q);
            }
        }
    }
    // Backward: uncompute carry out of bit i, then write sum bit i.
    for i in (0..n).rev() {
        let ci = carries[i];
        let cb = bit(c, i);
        if i + 1 < n {
            if let Some(co) = carries[i + 1] {
                match (ci, cb) {
                    (None, false) => unreachable!(),
                    (None, true) => match ctrl {
                        Some(t) => {
                            let m = b.alloc_bit();
                            b.hmr(co, m);
                            b.cz_if(t, v[i], m);
                        }
                        None => b.cx(v[i], co),
                    },
                    (Some(cq), false) => {
                        let m = b.alloc_bit();
                        b.hmr(co, m);
                        b.cz_if(v[i], cq, m);
                    }
                    (Some(cq), true) => match ctrl {
                        Some(t) => {
                            b.cx(cq, co);
                            b.cx(cq, t);
                            let m = b.alloc_bit();
                            b.hmr(co, m);
                            b.cz_if(t, v[i], m);
                            b.cx(cq, t);
                            // v[i] currently holds v ^ c_i; sum = that ^ a.
                        }
                        None => or_step_uncompute(b, v[i], cq, co),
                    },
                }
                b.free(co);
            }
        }
        // Sum bit: v_i ^= a_i ^ c_i (accounting for the in-place v ^= c_i of
        // the controlled-MAJ forward step).
        let modified = matches!((ci, cb, ctrl), (Some(_), true, Some(_))) && i + 1 < n;
        if cb {
            match ctrl {
                Some(t) => b.cx(t, v[i]),
                None => b.x(v[i]),
            }
        }
        if let Some(cq) = ci {
            if !modified {
                b.cx(cq, v[i]);
            }
        }
    }
    if subtract {
        for &q in v {
            b.x(q);
        }
    }
}
