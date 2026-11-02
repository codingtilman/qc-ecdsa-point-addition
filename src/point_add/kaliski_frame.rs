//! Lazy-frame Kaliski iteration: drops the STEP-9 swap-back of both
//! (u, v_w) and (r, s). A persistent `sig` qubit (zero at Kaliski start and,
//! once terminated, at the end) holds the current frame = original `a`; both
//! pairs are swapped by the relative swap `e` each iteration. The identity
//! below only uses true-frame invariants, so it holds in any representation.
//!
//! The original iteration swaps (u, v_w) and (r, s) by `a` (= "u is the
//! target"), updates, then swaps both pairs back. Here both pairs stay in the
//! post-swap frame: X (`u`) holds the non-target, Y (`v_w`) the halved
//! target, and (r, s) the post-swap coefficients. The Kaliski selection rule
//! is symmetric under (u,r) <-> (v,s), so the next iteration only needs the
//! relative swap `e` = "X is target".
//!
//! Per iteration (sig = current frame = previous true `a`):
//!   add    = f & X0 & Y0
//!   e      = (f & !X0) ^ (add & cmp),   cmp = X > Y, ties (X == Y == 1)
//!            broken by sig (comparator carry-in) so the true v is chosen.
//!   cswap(e: X,Y); cswap(e: r,s); sig ^= e (= true a)
//!   STEP 4 (Y -= add*X; s' += add*r')
//!   e ^= r'[0] & (sig | add)        (exact identity, see below)
//!   m_i ^= add; add ^= f & m_i (measured)
//!   STEP 6-8 (halve Y, double r')
//!
//! Identity for e: if sigma=1, r' = s_old so e = !sigma_old = s_old[0] = r'[0].
//! If sigma=0, s_old[0] = s_new[0] ^ (add & r[0]) = 1 ^ (add & r[0]), so
//! e = sigma_old = add & r[0].
//!
//! History bit m_i = term ^ add (term from STEP 0; disjoint with add, and
//! add == f_post & m_i).

use super::*;

/// `with_gt` whose carry-in is an existing qubit: flag = (u + cin > v).
fn with_gt_cin<F: FnOnce(&mut B)>(
    b: &mut B,
    u_gt: &[QubitId],
    v_lt: &[QubitId],
    flag: QubitId,
    c_in: QubitId,
    body: F,
) {
    // with_lt(v_lt, u_gt): carry of (~v + u + c_in) = (u + c_in > v).
    let u = v_lt;
    let v = u_gt;
    let n = u.len();
    assert_eq!(n, v.len());
    let carries = b.alloc_qubits(n);
    for i in 0..n {
        b.x(u[i]);
    }
    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
    }
    b.cx(u[n - 1], flag);
    body(b);
    b.cx(u[n - 1], flag);
    for i in (1..n).rev() {
        b.cx(carries[i], u[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(u[i - 1], v[i], m);
        b.cx(u[i], u[i - 1]);
        b.cx(u[i], v[i]);
    }
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);
    for i in 0..n {
        b.x(u[i]);
    }
    b.free_vec(&carries);
}

/// e ^= r[0] & (sig | add)
fn e_identity(b: &mut B, r0: QubitId, sig: QubitId, add_f: QubitId, e: QubitId) {
    let g = b.alloc_qubit();
    or_step(b, sig, add_f, g);
    b.ccx(r0, g, e);
    or_step_uncompute(b, sig, add_f, g);
    b.free(g);
}

fn step4_forward(b: &mut B, u: &[QubitId], v_w: &[QubitId], r: &[QubitId], s: &[QubitId], add_f: QubitId, iter_idx: usize) {
    let n = u.len();
    let tmp = b.alloc_qubits(n);
    let load_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    for i in 0..load_width {
        b.ccx(add_f, u[i], tmp[i]);
    }
    let tmp_sub_slice: Vec<QubitId> = tmp[0..load_width].to_vec();
    let v_w_sub_slice: Vec<QubitId> = v_w[0..load_width].to_vec();
    sub_nbit_qq_fast(b, &tmp_sub_slice, &v_w_sub_slice);
    let transform_width = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    for i in 0..transform_width {
        b.cx(r[i], u[i]);
    }
    for i in 0..transform_width {
        b.ccx(add_f, u[i], tmp[i]);
    }
    for i in 0..transform_width {
        b.cx(r[i], u[i]);
    }
    let add_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
    let mut tmp_slice: Vec<QubitId> = tmp[0..transform_width].to_vec();
    let tmp_pad = if add_width > transform_width {
        let q = b.alloc_qubit();
        tmp_slice.push(q);
        Some(q)
    } else {
        None
    };
    let s_slice: Vec<QubitId> = s[0..add_width].to_vec();
    add_nbit_qq_fast(b, &tmp_slice, &s_slice);
    if let Some(q) = tmp_pad {
        b.free(q);
    }
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(tmp[i], m);
        if i < transform_width {
            b.cz_if(add_f, r[i], m);
        } else if i < load_width {
            b.cz_if(add_f, u[i], m);
        }
    }
    b.free_vec(&tmp);
}

fn step4_backward(b: &mut B, u: &[QubitId], v_w: &[QubitId], r: &[QubitId], s: &[QubitId], add_f: QubitId, iter_idx: usize) {
    let n = u.len();
    let tmp = b.alloc_qubits(n);
    let load_width = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    for i in 0..load_width {
        b.ccx(add_f, r[i], tmp[i]);
    }
    let sub_width = if iter_idx + 2 < n { iter_idx + 2 } else { n };
    let tmp_sub_slice: Vec<QubitId> = tmp[0..sub_width].to_vec();
    let s_slice: Vec<QubitId> = s[0..sub_width].to_vec();
    sub_nbit_qq_fast(b, &tmp_sub_slice, &s_slice);
    let transform_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    for i in 0..transform_width {
        b.cx(r[i], u[i]);
    }
    for i in 0..transform_width {
        b.ccx(add_f, u[i], tmp[i]);
    }
    for i in 0..transform_width {
        b.cx(r[i], u[i]);
    }
    let add_width = transform_width;
    let tmp_add_slice: Vec<QubitId> = tmp[0..add_width].to_vec();
    let v_w_slice: Vec<QubitId> = v_w[0..add_width].to_vec();
    add_nbit_qq_fast(b, &tmp_add_slice, &v_w_slice);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(tmp[i], m);
        if i < transform_width {
            b.cz_if(add_f, u[i], m);
        } else if i < load_width {
            b.cz_if(add_f, r[i], m);
        }
    }
    b.free_vec(&tmp);
}

fn step0_forward(b: &mut B, v_w: &[QubitId], m_i: QubitId, f: QubitId, scratch: QubitId, iter_idx: usize) {
    let n = v_w.len();
    let or_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    with_eq_zero_fast(b, &v_w[0..or_width], scratch, |b| {
        b.ccx(f, scratch, m_i);
    });
    b.cx(m_i, f);
}

fn step0_backward(b: &mut B, v_w: &[QubitId], m_i: QubitId, f: QubitId, scratch: QubitId, iter_idx: usize) {
    let n = v_w.len();
    b.cx(m_i, f);
    let or_width = if iter_idx < n { n } else { 2 * n - iter_idx };
    with_eq_zero_fast(b, &v_w[0..or_width], scratch, |b| {
        b.ccx(f, scratch, m_i);
    });
}

/// Forward lazy-frame iteration. `f = None` means the bulk prefix (f = 1 and
/// no termination check).
pub(super) fn frame_iter_forward(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    r: &[QubitId],
    s: &[QubitId],
    m_i: QubitId,
    f: Option<QubitId>,
    sig: QubitId,
    iter_idx: usize,
) {
    let n = u.len();
    let w_uv = if iter_idx < n { n } else { 2 * n - iter_idx };
    let w_rs3 = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    let bulk = f.is_none();
    let saved = b.phase;

    let e = b.alloc_qubit();
    let add_f = b.alloc_qubit();

    if let Some(f) = f {
        b.set_phase("kal_step0_eqzero");
        step0_forward(b, v_w, m_i, f, add_f, iter_idx);
    }

    b.set_phase(if bulk { "kal_bulk_step1" } else { "kal_step1" });
    {
        let z = b.alloc_qubit();
        match f {
            Some(f) => b.ccx(f, u[0], z),
            None => b.cx(u[0], z),
        }
        b.ccx(z, v_w[0], add_f); // add = f & X0 & Y0
        match f {
            Some(f) => b.cx(f, e),
            None => b.x(e),
        }
        b.cx(z, e); // e = f & !X0
        match f {
            Some(f) => {
                let zm = b.alloc_bit();
                b.hmr(z, zm);
                b.cz_if(f, u[0], zm);
            }
            None => b.cx(u[0], z),
        }
        b.free(z);
    }

    b.set_phase(if bulk { "kal_bulk_step2" } else { "kal_step2" });
    {
        let l_gt = b.alloc_qubit();
        with_gt_cin(b, &u[..w_uv], &v_w[..w_uv], l_gt, sig, |b| {
            b.ccx(l_gt, add_f, e);
        });
        b.free(l_gt);
    }

    b.set_phase(if bulk { "kal_bulk_step3_cswap" } else { "kal_step3_cswap" });
    for j in 0..w_uv {
        cswap(b, e, u[j], v_w[j]);
    }
    for j in 0..w_rs3 {
        cswap(b, e, r[j], s[j]);
    }
    b.cx(e, sig);

    b.set_phase(if bulk { "kal_bulk_step4" } else { "kal_step4" });
    step4_forward(b, u, v_w, r, s, add_f, iter_idx);

    b.set_phase(if bulk { "kal_bulk_step5" } else { "kal_step5" });
    e_identity(b, r[0], sig, add_f, e);
    b.cx(add_f, m_i);
    match f {
        Some(f) => {
            let am = b.alloc_bit();
            b.hmr(add_f, am);
            b.cz_if(f, m_i, am);
        }
        None => b.cx(m_i, add_f),
    }

    b.set_phase(if bulk { "kal_bulk_step6_7_8" } else { "kal_step6_7_8" });
    for i in 0..(n - 1) {
        b.swap(v_w[i], v_w[i + 1]);
    }
    if iter_idx < r_small_threshold() {
        mod_double_no_corr(b, r);
    } else {
        mod_double_inplace_fast(b, r, p);
    }

    b.free(add_f);
    b.free(e);
    b.set_phase(saved);
}

/// Exact reverse of `frame_iter_forward`.
pub(super) fn frame_iter_backward(
    b: &mut B,
    p: U256,
    u: &[QubitId],
    v_w: &[QubitId],
    r: &[QubitId],
    s: &[QubitId],
    m_i: QubitId,
    f: Option<QubitId>,
    sig: QubitId,
    iter_idx: usize,
) {
    let n = u.len();
    let w_uv = if iter_idx < n { n } else { 2 * n - iter_idx };
    let w_rs3 = if iter_idx + 1 < n { iter_idx + 1 } else { n };
    let bulk = f.is_none();
    let saved = b.phase;

    let e = b.alloc_qubit();
    let add_f = b.alloc_qubit();

    b.set_phase(if bulk { "bk_bulk_step6_7_8" } else { "bk_step6_7_8" });
    if iter_idx < r_small_threshold() {
        mod_halve_no_corr(b, r);
    } else {
        mod_halve_inplace_fast(b, r, p);
    }
    for i in (0..(n - 1)).rev() {
        b.swap(v_w[i], v_w[i + 1]);
    }

    b.set_phase(if bulk { "bk_bulk_step5" } else { "bk_step5" });
    match f {
        Some(f) => b.ccx(f, m_i, add_f),
        None => b.cx(m_i, add_f),
    }
    b.cx(add_f, m_i);
    e_identity(b, r[0], sig, add_f, e);

    b.set_phase(if bulk { "bk_bulk_step4" } else { "bk_step4" });
    step4_backward(b, u, v_w, r, s, add_f, iter_idx);

    b.set_phase(if bulk { "bk_bulk_step3_cswap" } else { "bk_step3_cswap" });
    b.cx(e, sig);
    for j in (0..w_rs3).rev() {
        cswap(b, e, r[j], s[j]);
    }
    for j in (0..w_uv).rev() {
        cswap(b, e, u[j], v_w[j]);
    }

    b.set_phase(if bulk { "bk_bulk_step2" } else { "bk_step2" });
    {
        let l_gt = b.alloc_qubit();
        with_gt_cin(b, &u[..w_uv], &v_w[..w_uv], l_gt, sig, |b| {
            b.ccx(l_gt, add_f, e);
        });
        b.free(l_gt);
    }

    b.set_phase(if bulk { "bk_bulk_step1" } else { "bk_step1" });
    {
        let z = b.alloc_qubit();
        match f {
            Some(f) => b.ccx(f, u[0], z),
            None => b.cx(u[0], z),
        }
        b.cx(z, e);
        match f {
            Some(f) => b.cx(f, e),
            None => b.x(e),
        }
        b.ccx(z, v_w[0], add_f);
        match f {
            Some(f) => {
                let zm = b.alloc_bit();
                b.hmr(z, zm);
                b.cz_if(f, u[0], zm);
            }
            None => b.cx(u[0], z),
        }
        b.free(z);
    }
    if let Some(f) = f {
        b.set_phase("bk_step0_eqzero");
        step0_backward(b, v_w, m_i, f, add_f, iter_idx);
    }

    b.free(add_f);
    b.free(e);
    b.set_phase(saved);
}
