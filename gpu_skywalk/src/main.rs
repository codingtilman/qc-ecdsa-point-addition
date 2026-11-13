use alloy_primitives::U256;
use quantum_ecc::circuit::{
    analyze_ops, BitId, Op, OperationType as K, QubitId, QubitOrBit, RegisterId,
};
use quantum_ecc::sim::Simulator;
use quantum_ecc::weierstrass_elliptic_curve::WeierstrassEllipticCurve;
use sha3::{digest::{ExtendableOutput, Update, XofReader}, Shake256};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Instant;

const OPS_MAGIC: &[u8; 8] = b"QECCOPSZ";
const PACK_MAGIC: &[u8; 8] = b"SKYGPU1\0";
const INPUT_MAGIC: &[u8; 8] = b"SKYINP1\0";
const DEFAULT_INPUT_DOMAIN: &[u8] = b"ecdsafail-independent-paired-v1";
const DEFAULT_MEASUREMENT_DOMAIN: &[u8] = b"ecdsafail-independent-paired-measurements-v1";
const OP_BYTES: usize = 56;
const NO16: u16 = u16::MAX;

#[derive(Clone, Debug)]
struct CensusSite {
    op: usize,
    phase: String,
    raw: usize,
    site: usize,
    emax: usize,
    tg: Vec<u64>,
    src: Vec<u64>,
    sign: Vec<u64>,
    exp: Vec<u64>,
}

fn scalar_field(line: &str, key: &str) -> usize {
    let tail = &line[line.find(key).unwrap_or_else(|| panic!("missing {key} in {line}")) + key.len()..];
    tail.bytes().take_while(u8::is_ascii_digit).fold(0usize, |v, d| v * 10 + (d - b'0') as usize)
}

fn text_field<'a>(line: &'a str, key: &str) -> &'a str {
    let tail = &line[line.find(key).unwrap_or_else(|| panic!("missing {key} in {line}")) + key.len()..];
    &tail[..tail.find(' ').unwrap_or(tail.len())]
}

fn vec_field(line: &str, key: &str) -> Vec<u64> {
    let tail = &line[line.find(key).unwrap_or_else(|| panic!("missing {key} in {line}")) + key.len()..];
    let body = &tail[..tail.find(']').unwrap()];
    if body.is_empty() { return Vec::new(); }
    body.split(',').map(|v| v.trim().parse().unwrap()).collect()
}

fn load_census_sites(path: &str, ops: &[Op]) -> Vec<CensusSite> {
    let text = std::fs::read_to_string(path).unwrap();
    let mut out = Vec::new();
    for line in text.lines().filter(|line| line.contains("R125_CENSUS dir=F")) {
        out.push(CensusSite {
            op: scalar_field(line, "op="), phase: text_field(line, "phase=").to_owned(),
            raw: scalar_field(line, "raw_t="),
            site: scalar_field(line, "site_t="), emax: scalar_field(line, "emax="),
            tg: vec_field(line, "tg=["), src: vec_field(line, "src=["),
            sign: vec_field(line, "s=["), exp: vec_field(line, "k=["),
        });
    }
    out.sort_by_key(|x| x.op);
    assert_eq!(out.len(), 123, "expected the 123 forward R125 sites");
    let mut depth = 0usize;
    let mut next = 0usize;
    for (i, op) in ops.iter().enumerate() {
        while next < out.len() && out[next].op == i {
            assert_eq!(depth, 0, "census boundary op {i} is inside condition depth {depth}");
            assert_eq!(out[next].tg.len(), 256);
            assert_eq!(out[next].src.len(), 256);
            assert_eq!(out[next].sign.len(), 3);
            assert_eq!(out[next].exp.len(), 3);
            next += 1;
        }
        match op.kind {
            K::PushCondition => depth += 1,
            K::PopCondition => depth = depth.checked_sub(1).unwrap(),
            _ => {}
        }
    }
    assert_eq!(next, out.len());
    assert_eq!(depth, 0);
    eprintln!("CENSUS metadata={} forward_sites={} boundaries_depth0=yes", path, out.len());
    out
}

fn q_u256<R: XofReader>(sim: &Simulator<'_, R>, ids: &[u64], shot: usize) -> U256 {
    let mut out = U256::ZERO;
    for (i, &id) in ids.iter().enumerate() {
        if ((sim.qubit(QubitId(id)) >> shot) & 1) != 0 { out.set_bit(i, true); }
    }
    out
}

fn q_bool<R: XofReader>(sim: &Simulator<'_, R>, id: u64, shot: usize) -> bool {
    // NO_QUBIT and R125's Q_E3_GHOST both encode an absent, known-zero k3.
    id < sim.num_qubits as u64 && ((sim.qubit(QubitId(id)) >> shot) & 1) != 0
}

/// Exact integer predicate used by the R125 fold core: true iff a carry leaves
/// the selected low window. This mirrors y15_a2::tests::model_fwd without
/// depending on test-only challenge code.
fn fold_escapes(tv: U256, sv: U256, sign: [bool; 3], e: usize, mw: usize) -> bool {
    let sp = |b: bool| if b { U256::MAX - sv } else { sv };
    let (mut lo, mut top) = (tv, 0u64);
    for (i, &si) in sign.iter().enumerate() {
        let v = sp(si);
        let (a, cy) = lo.overflowing_add(v << i);
        lo = a;
        top += u64::from(cy) + if i > 0 { (v >> (256 - i)).to::<u64>() } else { 0 };
    }
    let ev = top as i128;
    let nmv = (sign[0] as i128) + 2 * (sign[1] as i128) + 4 * (sign[2] as i128);
    let kk = 3 + e;
    let f1 = (1i128 << 32) + 976;
    let lw = (lo & ((U256::from(1u8) << mw) - U256::from(1u8))).to::<u128>() as i128;
    let base = lw + ev - (nmv - ev) * f1;
    let mk = 1i128 << kk;
    let mut finv = 1i128;
    for _ in 0..6 { finv = (finv * (2 - (f1 + 1) * finv)).rem_euclid(1i128 << 20); }
    let m = (base.rem_euclid(mk) * finv.rem_euclid(mk)).rem_euclid(mk);
    let wtrue = base - m * f1;
    wtrue < 0 || wtrue >= (1i128 << mw)
}

fn census_masks<R: XofReader>(sim: &Simulator<'_, R>, site: &CensusSite, bs: usize) -> [u64; 4] {
    let mut masks = [0u64; 4];
    for shot in 0..bs {
        let tv = q_u256(sim, &site.tg, shot);
        let sv = q_u256(sim, &site.src, shot);
        let sign = [q_bool(sim, site.sign[0], shot), q_bool(sim, site.sign[1], shot), q_bool(sim, site.sign[2], shot)];
        let e = usize::from(q_bool(sim, site.exp[0], shot))
            + 2 * usize::from(q_bool(sim, site.exp[1], shot))
            + 4 * usize::from(q_bool(sim, site.exp[2], shot));
        assert!(e <= site.emax, "site {} exponent {e} exceeds emax {}", site.site, site.emax);
        for (j, mw) in (55..=58).enumerate() {
            if fold_escapes(tv, sv, sign, e, mw) { masks[j] |= 1u64 << shot; }
        }
    }
    masks
}

fn kind(v: u32) -> K {
    match v {
        0 => K::Neg, 1 => K::Register, 2 => K::AppendToRegister, 3 => K::BitInvert,
        4 => K::BitStore0, 5 => K::BitStore1, 6 => K::X, 7 => K::Z, 8 => K::CX,
        9 => K::CZ, 10 => K::Swap, 11 => K::R, 12 => K::Hmr, 13 => K::CCX,
        14 => K::CCZ, 15 => K::PushCondition, 16 => K::PopCondition,
        17 => K::DebugPrint, _ => panic!("bad op kind {v}"),
    }
}

fn u64_at(rec: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(rec[off..off + 8].try_into().unwrap())
}

fn load_ops(path: &str) -> Vec<Op> {
    let mut f = File::open(path).unwrap();
    let mut h = [0u8; 16];
    f.read_exact(&mut h).unwrap();
    assert_eq!(&h[..8], OPS_MAGIC);
    let n = u64::from_le_bytes(h[8..16].try_into().unwrap()) as usize;
    let mut z = zstd::stream::read::Decoder::new(BufReader::new(f)).unwrap();
    z.window_log_max(27).unwrap();
    let mut ops = Vec::with_capacity(n);
    let mut rec = [0u8; OP_BYTES];
    for _ in 0..n {
        z.read_exact(&mut rec).unwrap();
        let op = Op {
            kind: kind(u32::from_le_bytes(rec[..4].try_into().unwrap())),
            q_control2: QubitId(u64_at(&rec, 8)), q_control1: QubitId(u64_at(&rec, 16)),
            q_target: QubitId(u64_at(&rec, 24)), c_target: BitId(u64_at(&rec, 32)),
            c_condition: BitId(u64_at(&rec, 40)), r_target: RegisterId(u64_at(&rec, 48)),
        };
        op.validate();
        ops.push(op);
    }
    ops
}

fn curve() -> WeierstrassEllipticCurve {
    let h = |s: &str| U256::from_str_radix(s, 16).unwrap();
    WeierstrassEllipticCurve {
        modulus: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"),
        a: U256::ZERO, b: U256::from(7),
        gx: h("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
        gy: h("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"),
        order: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141"),
    }
}

type Input = ((U256, U256), (U256, U256), (U256, U256));

fn input_domain() -> Vec<u8> {
    std::env::var("SKYWALK_INPUT_DOMAIN").map_or_else(|_| DEFAULT_INPUT_DOMAIN.to_vec(), String::into_bytes)
}

fn measurement_domain() -> Vec<u8> {
    std::env::var("SKYWALK_MEASUREMENT_DOMAIN").map_or_else(|_| DEFAULT_MEASUREMENT_DOMAIN.to_vec(), String::into_bytes)
}

fn inputs(n: usize) -> Vec<Input> {
    let c = curve();
    let mut h = Shake256::default();
    h.update(&input_domain());
    let mut x = h.finalize_xof();
    let threads = std::env::var("SKYWALK_INPUT_THREADS").ok()
        .and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).max(1);
    if threads > 1 {
        // SHAKE remains serial and byte-identical. Only the independent affine
        // point calculations are distributed, with results restored in scalar order.
        // Extra candidates preserve the sequential generator's skip semantics for
        // the astronomically rare invalid/equal point pair.
        let candidates = n.checked_add(16).unwrap();
        let mut scalars = Vec::with_capacity(candidates);
        for _ in 0..candidates {
            let mut a = [0u8; 32];
            let mut b = [0u8; 32];
            XofReader::read(&mut x, &mut a);
            XofReader::read(&mut x, &mut b);
            scalars.push((U256::from_le_bytes(a), U256::from_le_bytes(b)));
        }
        let zero = ((U256::ZERO, U256::ZERO), (U256::ZERO, U256::ZERO), (U256::ZERO, U256::ZERO));
        let mut computed = vec![zero; candidates];
        let chunk = candidates.div_ceil(threads);
        std::thread::scope(|scope| {
            for (dst, src) in computed.chunks_mut(chunk).zip(scalars.chunks(chunk)) {
                let c = c.clone();
                scope.spawn(move || {
                    for (slot, &(a, b)) in dst.iter_mut().zip(src) {
                        let p = c.mul(c.gx, c.gy, a);
                        let q = c.mul(c.gx, c.gy, b);
                        *slot = (p, q, c.add(p.0, p.1, q.0, q.1));
                    }
                });
            }
        });
        let out: Vec<Input> = computed.into_iter().filter(|(p, q, _)| {
            p.0 != q.0 && !(p.0.is_zero() && p.1.is_zero()) && !(q.0.is_zero() && q.1.is_zero())
        }).take(n).collect();
        assert_eq!(out.len(), n, "parallel input candidate reserve exhausted");
        return out;
    }
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        XofReader::read(&mut x, &mut a);
        XofReader::read(&mut x, &mut b);
        let p = c.mul(c.gx, c.gy, U256::from_le_bytes(a));
        let q = c.mul(c.gx, c.gy, U256::from_le_bytes(b));
        if p.0 == q.0 || (p.0.is_zero() && p.1.is_zero()) || (q.0.is_zero() && q.1.is_zero()) {
            continue;
        }
        out.push((p, q, c.add(p.0, p.1, q.0, q.1)));
    }
    out
}

fn input_domain_id() -> [u8; 32] {
    let mut h = Shake256::default();
    h.update(b"skywalk-input-cache-domain-v1\0");
    h.update(&input_domain());
    let mut x = h.finalize_xof();
    let mut out = [0u8; 32];
    XofReader::read(&mut x, &mut out);
    out
}

fn write_input_cache(path: &str, values: &[Input]) {
    let mut w = BufWriter::new(File::create(path).unwrap());
    w.write_all(INPUT_MAGIC).unwrap();
    w.write_all(&(values.len() as u64).to_le_bytes()).unwrap();
    w.write_all(&input_domain_id()).unwrap();
    for &(p, q, e) in values {
        for v in [p.0, p.1, q.0, q.1, e.0, e.1] {
            w.write_all(&v.to_le_bytes::<32>()).unwrap();
        }
    }
    w.flush().unwrap();
}

fn read_input_cache(path: &str, n: usize) -> Vec<Input> {
    let mut r = BufReader::new(File::open(path).unwrap());
    let mut magic = [0u8; 8];
    r.read_exact(&mut magic).unwrap();
    assert_eq!(&magic, INPUT_MAGIC, "bad input-cache magic");
    let mut count = [0u8; 8];
    r.read_exact(&mut count).unwrap();
    let count = u64::from_le_bytes(count) as usize;
    assert!(count >= n, "input cache has {count} entries, need {n}");
    let mut domain = [0u8; 32];
    r.read_exact(&mut domain).unwrap();
    assert_eq!(domain, input_domain_id(), "input cache belongs to a different corpus");
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let mut vals = [U256::ZERO; 6];
        for v in &mut vals {
            let mut bytes = [0u8; 32];
            r.read_exact(&mut bytes).unwrap();
            *v = U256::from_le_bytes(bytes);
        }
        out.push(((vals[0], vals[1]), (vals[2], vals[3]), (vals[4], vals[5])));
    }
    out
}

fn reg_from_state(state: &[u64], nq: usize, reg: &[QubitOrBit], shot: usize) -> U256 {
    let mut out = U256::ZERO;
    for (i, wire) in reg.iter().enumerate() {
        let word = match *wire {
            QubitOrBit::Qubit(q) => state[q.0 as usize],
            QubitOrBit::Bit(b) => state[nq + b.0 as usize],
        };
        out.set_bit(i, ((word >> shot) & 1) != 0);
    }
    out
}

/// Recover the immutable paired-input corpus from an existing validated pack.
/// This is a one-time migration path: later candidates read the compact raw cache
/// and only redo cheap bit-slicing for their own wire layout.
fn extract_input_cache(pack_path: &str, n: usize, regs: &[Vec<QubitOrBit>]) -> Vec<Input> {
    let mut r = BufReader::new(File::open(pack_path).unwrap());
    let mut magic = [0u8; 8];
    r.read_exact(&mut magic).unwrap();
    assert_eq!(&magic, PACK_MAGIC, "bad source-pack magic");
    let mut h = [0u64; 10];
    for x in &mut h {
        let mut b = [0u8; 8];
        r.read_exact(&mut b).unwrap();
        *x = u64::from_le_bytes(b);
    }
    let [version, nops, nq, nb, shots, batches, _nmeas, nout, ndecl, depth] = h;
    assert_eq!(version, 1);
    assert!(depth <= 4 && shots as usize >= n && nout == 512);
    assert_eq!(regs.len(), 4);
    let skip = nops * 16 + (nout + ndecl) * 2;
    r.seek(SeekFrom::Current(skip.try_into().unwrap())).unwrap();
    let stride = (nq + nb) as usize;
    let mut initial = vec![0u64; batches as usize * stride];
    for x in &mut initial {
        let mut b = [0u8; 8]; r.read_exact(&mut b).unwrap(); *x = u64::from_le_bytes(b);
    }
    let mut expected = vec![0u64; batches as usize * nout as usize];
    for x in &mut expected {
        let mut b = [0u8; 8]; r.read_exact(&mut b).unwrap(); *x = u64::from_le_bytes(b);
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let batch = i / 64;
        let shot = i % 64;
        let state = &initial[batch * stride..(batch + 1) * stride];
        let p = (reg_from_state(state, nq as usize, &regs[0], shot),
                 reg_from_state(state, nq as usize, &regs[1], shot));
        let q = (reg_from_state(state, nq as usize, &regs[2], shot),
                 reg_from_state(state, nq as usize, &regs[3], shot));
        let row = &expected[batch * 512..(batch + 1) * 512];
        let mut ex = U256::ZERO; let mut ey = U256::ZERO;
        for bit in 0..256 {
            ex.set_bit(bit, ((row[bit] >> shot) & 1) != 0);
            ey.set_bit(bit, ((row[256 + bit] >> shot) & 1) != 0);
        }
        out.push((p, q, (ex, ey)));
    }
    out
}

fn cached_inputs(n: usize, regs: &[Vec<QubitOrBit>]) -> Vec<Input> {
    if let Ok(path) = std::env::var("SKYWALK_INPUT_CACHE") {
        if Path::new(&path).exists() {
            return read_input_cache(&path, n);
        }
        if let Ok(pack) = std::env::var("SKYWALK_INPUT_CACHE_FROM_PACK") {
            let out = extract_input_cache(&pack, n, regs);
            write_input_cache(&path, &out);
            return out;
        }
    }
    let out = inputs(n);
    if let Ok(path) = std::env::var("SKYWALK_WRITE_INPUT_CACHE") {
        write_input_cache(&path, &out);
    }
    out
}

// Byte-oriented SHAKE256 copied from the MIT-licensed ecdsafail_gpu_toolkit's
// dump_gpu_state.rs, then extended with state snapshots and O(blocks) skipping.
const ROTC: [u32; 24] = [1,3,6,10,15,21,28,36,45,55,2,14,27,41,56,8,25,43,62,18,39,61,20,44];
const PILN: [usize; 24] = [10,7,11,17,18,3,5,16,8,21,24,4,15,23,19,13,12,2,20,14,22,9,6,1];
const RNDC: [u64; 24] = [
    0x1,0x8082,0x800000000000808a,0x8000000080008000,0x808b,0x80000001,
    0x8000000080008081,0x8000000000008009,0x8a,0x88,0x80008009,0x8000000a,
    0x8000808b,0x800000000000008b,0x8000000000008089,0x8000000000008003,
    0x8000000000008002,0x8000000000000080,0x800a,0x800000008000000a,
    0x8000000080008081,0x8000000000008080,0x80000001,0x8000000080008008,
];

fn keccakf(st: &mut [u64; 25]) {
    for &rc in &RNDC {
        let mut bc = [0u64; 5];
        for i in 0..5 { bc[i] = st[i] ^ st[i+5] ^ st[i+10] ^ st[i+15] ^ st[i+20]; }
        for i in 0..5 {
            let t = bc[(i+4)%5] ^ bc[(i+1)%5].rotate_left(1);
            for j in (0..25).step_by(5) { st[j+i] ^= t; }
        }
        let mut t = st[1];
        for i in 0..24 {
            let j = PILN[i];
            let z = st[j];
            st[j] = t.rotate_left(ROTC[i]);
            t = z;
        }
        for j in (0..25).step_by(5) {
            let row = [st[j], st[j+1], st[j+2], st[j+3], st[j+4]];
            for i in 0..5 { st[j+i] ^= (!row[(i+1)%5]) & row[(i+2)%5]; }
        }
        st[0] ^= rc;
    }
}

#[derive(Clone)]
struct Squeeze { st: [u64; 25], pt: usize }

impl Squeeze {
    fn new() -> Self { Self { st: [0u64; 25], pt: 0 } }

    fn absorb(&mut self, data: &[u8]) {
        for &b in data {
            let lane = self.pt >> 3;
            self.st[lane] ^= (b as u64) << (8 * (self.pt & 7));
            self.pt += 1;
            if self.pt == 136 { keccakf(&mut self.st); self.pt = 0; }
        }
    }

    fn finalize(mut self) -> Self {
        let lane = self.pt >> 3;
        self.st[lane] ^= 0x1fu64 << (8 * (self.pt & 7));
        self.st[16] ^= 0x80u64 << 56;
        keccakf(&mut self.st);
        self.pt = 0;
        self
    }

    fn from_tag(tag: &[u8]) -> Self {
        let mut out = Self::new();
        out.absorb(tag);
        out.finalize()
    }

    fn next_u64(&mut self) -> u64 {
        if self.pt == 136 { keccakf(&mut self.st); self.pt = 0; }
        let out = self.st[self.pt >> 3];
        self.pt += 8;
        out
    }

    fn advance_words(&mut self, mut words: usize) {
        while words != 0 {
            if self.pt == 136 { keccakf(&mut self.st); self.pt = 0; }
            let avail = (136 - self.pt) / 8;
            let take = words.min(avail);
            self.pt += take * 8;
            words -= take;
        }
    }
}

const OFFICIAL_SHOTS: usize = 9024;

fn absorb_op(s: &mut Squeeze, op: &Op, q_target: u64) {
    s.absorb(&[op.kind as u8]);
    s.absorb(&op.q_control2.0.to_le_bytes());
    s.absorb(&op.q_control1.0.to_le_bytes());
    s.absorb(&q_target.to_le_bytes());
    s.absorb(&op.c_target.0.to_le_bytes());
    s.absorb(&op.c_condition.0.to_le_bytes());
    s.absorb(&op.r_target.0.to_le_bytes());
}

fn fiat_prefix(ops: &[Op]) -> Squeeze {
    assert!(ops.len() >= 96);
    let tail = ops.len() - 96;
    let mut s = Squeeze::new();
    s.absorb(b"quantum_ecc-fiat-shamir-v2");
    s.absorb(&(ops.len() as u64).to_le_bytes());
    for op in &ops[..tail] { absorb_op(&mut s, op, op.q_target.0); }
    s
}

fn fiat_squeeze_from_prefix(mut s: Squeeze, ops: &[Op], nonce: u64) -> Squeeze {
    let tail = ops.len() - 96;
    for (j, op) in ops[tail..].iter().enumerate() {
        absorb_op(&mut s, op, (nonce >> (j / 2)) & 1);
    }
    s.finalize()
}

fn fiat_squeeze(ops: &[Op], nonce: u64) -> Squeeze {
    fiat_squeeze_from_prefix(fiat_prefix(ops), ops, nonce)
}

fn fiat_reader(ops: &[Op], nonce: u64) -> sha3::Shake256Reader {
    assert!(ops.len() >= 96);
    let tail = ops.len() - 96;
    let mut h = Shake256::default();
    h.update(b"quantum_ecc-fiat-shamir-v2");
    h.update(&(ops.len() as u64).to_le_bytes());
    for (i, op) in ops.iter().enumerate() {
        h.update(&[op.kind as u8]);
        h.update(&op.q_control2.0.to_le_bytes());
        h.update(&op.q_control1.0.to_le_bytes());
        let qt = if i >= tail { (nonce >> ((i - tail) / 2)) & 1 } else { op.q_target.0 };
        h.update(&qt.to_le_bytes());
        h.update(&op.c_target.0.to_le_bytes());
        h.update(&op.c_condition.0.to_le_bytes());
        h.update(&op.r_target.0.to_le_bytes());
    }
    h.finalize_xof()
}

fn nonce_corpus(ops: &[Op], nonces: &[u64], kept_shots: usize, nmeas: usize, threads: usize) -> (Vec<Input>, Vec<Squeeze>) {
    assert!(kept_shots > 0 && kept_shots <= OFFICIAL_SHOTS && kept_shots % 64 == 0);
    assert!(!nonces.is_empty());
    let count = nonces.len();
    let prefix = fiat_prefix(ops);
    let mut probe_a = fiat_squeeze_from_prefix(prefix.clone(), ops, nonces[0]);
    let mut probe_b = fiat_reader(ops, nonces[0]);
    for _ in 0..64 {
        let mut z = [0u8; 8];
        XofReader::read(&mut probe_b, &mut z);
        assert_eq!(probe_a.next_u64(), u64::from_le_bytes(z), "custom Fiat-Shamir SHAKE mismatch");
    }
    let zero_scalar = (U256::ZERO, U256::ZERO);
    let mut scalars = vec![zero_scalar; count * kept_shots];
    let batches_per_nonce = kept_shots / 64;
    let mut states = vec![prefix.clone(); count * batches_per_nonce];
    let nonce_chunk = count.div_ceil(threads.max(1));
    std::thread::scope(|scope| {
        for (worker, (scalar_dst, state_dst)) in scalars.chunks_mut(nonce_chunk * kept_shots)
            .zip(states.chunks_mut(nonce_chunk * batches_per_nonce)).enumerate()
        {
            let prefix = prefix.clone();
            scope.spawn(move || {
                let local_count = scalar_dst.len() / kept_shots;
                for local in 0..local_count {
                    let nonce = nonces[worker * nonce_chunk + local];
                    let mut sq = fiat_squeeze_from_prefix(prefix.clone(), ops, nonce);
                    for shot in 0..OFFICIAL_SHOTS {
                        let mut a = [0u8; 32];
                        let mut b = [0u8; 32];
                        for word in a.chunks_exact_mut(8) { word.copy_from_slice(&sq.next_u64().to_le_bytes()); }
                        for word in b.chunks_exact_mut(8) { word.copy_from_slice(&sq.next_u64().to_le_bytes()); }
                        if shot < kept_shots {
                            scalar_dst[local * kept_shots + shot] = (U256::from_le_bytes(a), U256::from_le_bytes(b));
                        }
                    }
                    for slot in &mut state_dst[local * batches_per_nonce..(local + 1) * batches_per_nonce] {
                        *slot = sq.clone();
                        sq.advance_words(nmeas);
                    }
                }
            });
        }
    });
    let c = curve();
    let zero = ((U256::ZERO, U256::ZERO), (U256::ZERO, U256::ZERO), (U256::ZERO, U256::ZERO));
    let mut out = vec![zero; scalars.len()];
    let chunk = scalars.len().div_ceil(threads.max(1));
    std::thread::scope(|scope| {
        for (dst, src) in out.chunks_mut(chunk).zip(scalars.chunks(chunk)) {
            let c = c.clone();
            scope.spawn(move || {
                for (slot, &(a, b)) in dst.iter_mut().zip(src) {
                    let p = c.mul(c.gx, c.gy, a);
                    let q = c.mul(c.gx, c.gy, b);
                    assert!(p.0 != q.0 && !(p.0.is_zero() && p.1.is_zero()) && !(q.0.is_zero() && q.1.is_zero()),
                        "rare official-corpus skip requires variable-length nonce packing");
                    *slot = (p, q, c.add(p.0, p.1, q.0, q.1));
                }
            });
        }
    });
    (out, states)
}

fn id16(x: u64) -> u16 {
    if x == u64::MAX { NO16 } else { u16::try_from(x).unwrap() }
}

fn wire16(x: &QubitOrBit) -> u16 {
    match *x {
        QubitOrBit::Qubit(q) => { assert!(q.0 < 0x8000); q.0 as u16 }
        QubitOrBit::Bit(b) => { assert!(b.0 < 0x8000); 0x8000 | b.0 as u16 }
    }
}

fn set_reg(q: &mut [u64], bits: &mut [u64], reg: &[QubitOrBit], v: U256, shot: usize) {
    for (i, x) in reg.iter().enumerate() {
        if v.bit(i) {
            match *x {
                QubitOrBit::Qubit(id) => q[id.0 as usize] |= 1u64 << shot,
                QubitOrBit::Bit(id) => bits[id.0 as usize] |= 1u64 << shot,
            }
        }
    }
}

fn write_u64(w: &mut impl Write, x: u64) { w.write_all(&x.to_le_bytes()).unwrap(); }
fn write_u16(w: &mut impl Write, x: u16) { w.write_all(&x.to_le_bytes()).unwrap(); }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(args.len() == 4, "usage: skywalk_gpu_pack OPS.bin N OUT.pack");
    let path = &args[1];
    let requested_n: usize = args[2].parse().unwrap();
    let out_path = &args[3];
    let all_start = Instant::now();
    let input_tag = input_domain();
    let measurement_tag = measurement_domain();
    let input_id_hex: String = input_domain_id().iter().map(|b| format!("{b:02x}")).collect();
    eprintln!("DOMAINS input={} measurement={} input_id={}", String::from_utf8_lossy(&input_tag),
        String::from_utf8_lossy(&measurement_tag), input_id_hex);

    let t = Instant::now();
    let ops = load_ops(path);
    let (nq, nb, _, regs) = analyze_ops(ops.iter());
    assert_eq!(regs.len(), 4);
    assert!(nq < NO16 as u64 && nb < NO16 as u64);
    let load_s = t.elapsed().as_secs_f64();
    let nmeas = ops.iter().filter(|o| matches!(o.kind, K::Hmr | K::R)).count();
    let nonce_cfg = if let Ok(list) = std::env::var("SKYWALK_NONCE_LIST") {
        let values: Vec<u64> = list.split(',').filter(|v| !v.is_empty())
            .map(|v| v.parse::<u64>().unwrap()).collect();
        assert!(!values.is_empty(), "SKYWALK_NONCE_LIST must not be empty");
        Some(values)
    } else {
        std::env::var("SKYWALK_NONCE_START").ok().map(|v| {
            let start = v.parse::<u64>().unwrap();
            let count = std::env::var("SKYWALK_NONCE_COUNT").unwrap_or_else(|_| "1".to_owned()).parse::<usize>().unwrap();
            (start..start.checked_add(count as u64).unwrap()).collect()
        })
    };
    let nonce_shots = std::env::var("SKYWALK_NONCE_SHOTS").ok()
        .map_or(OFFICIAL_SHOTS, |v| v.parse::<usize>().unwrap());
    let n = nonce_cfg.as_ref().map_or(requested_n, |nonces| nonces.len() * nonce_shots);
    assert_eq!(requested_n, n, "N must equal SKYWALK_NONCE_COUNT * SKYWALK_NONCE_SHOTS in nonce mode");

    let t = Instant::now();
    let threads = std::env::var("SKYWALK_INPUT_THREADS").ok()
        .and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).max(1);
    let (ins, nonce_states) = if let Some(nonces) = nonce_cfg.as_ref() {
        eprintln!("NONCE_CORPUS count={} first={} last={} shots_per_nonce={nonce_shots} official_shots={OFFICIAL_SHOTS}",
            nonces.len(), nonces[0], nonces[nonces.len() - 1]);
        let (ins, states) = nonce_corpus(&ops, nonces, nonce_shots, nmeas, threads);
        (ins, Some(states))
    } else {
        (cached_inputs(n, &regs), None)
    };
    let input_s = t.elapsed().as_secs_f64();
    if std::env::var("SKYWALK_PREP_ONLY").is_ok_and(|v| v == "1") {
        let mut digest = 0u64;
        for &(p, q, e) in &ins {
            for v in [p.0, p.1, q.0, q.1, e.0, e.1] {
                let b = v.to_le_bytes::<32>();
                for word in b.chunks_exact(8) {
                    digest = digest.rotate_left(7) ^ u64::from_le_bytes(word.try_into().unwrap());
                }
            }
        }
        eprintln!("PREP_ONLY shots={n} input_prep={input_s:.3}s digest={digest:016x}");
        return;
    }
    let batches = (n + 63) / 64;
    let max_depth = {
        let (mut d, mut m) = (0usize, 0usize);
        for o in &ops { if o.kind == K::PushCondition { d += 1; m = m.max(d); } else if o.kind == K::PopCondition { d -= 1; } }
        assert_eq!(d, 0); m
    };
    assert!(max_depth <= 4);

    let stride = (nq + nb) as usize;
    let mut initial = vec![0u64; batches * stride];
    let out_regs: Vec<u16> = regs[0].iter().chain(regs[1].iter()).map(wire16).collect();
    let mut expected = vec![0u64; batches * out_regs.len()];
    for b in 0..batches {
        let bs = 64.min(n - b * 64);
        let state = &mut initial[b*stride..(b+1)*stride];
        let (q, bits) = state.split_at_mut(nq as usize);
        for s in 0..bs {
            let (p, o, e) = ins[b*64+s];
            set_reg(q, bits, &regs[0], p.0, s); set_reg(q, bits, &regs[1], p.1, s);
            set_reg(q, bits, &regs[2], o.0, s); set_reg(q, bits, &regs[3], o.1, s);
            for i in 0..256 { if e.0.bit(i) { expected[b*out_regs.len()+i] |= 1u64 << s; } }
            for i in 0..256 { if e.1.bit(i) { expected[b*out_regs.len()+256+i] |= 1u64 << s; } }
        }
    }
    let mut declared: Vec<u16> = regs.iter().flat_map(|r| r.iter()).filter_map(|x| match x {
        QubitOrBit::Qubit(q) => Some(id16(q.0)), _ => None,
    }).collect();
    declared.sort_unstable(); declared.dedup();

    // Validate the copied SHAKE implementation before using snapshots.
    let mut ours = Squeeze::from_tag(&measurement_tag);
    let mut sh = Shake256::default(); sh.update(&measurement_tag);
    let mut sx = sh.finalize_xof();
    for _ in 0..64 {
        let mut z = [0u8; 8]; XofReader::read(&mut sx, &mut z);
        assert_eq!(ours.next_u64(), u64::from_le_bytes(z));
    }
    let states = if let Some(states) = nonce_states {
        assert_eq!(states.len(), batches);
        states
    } else {
        let mut states = Vec::with_capacity(batches);
        let mut sq = Squeeze::from_tag(&measurement_tag);
        for _ in 0..batches { states.push(sq.clone()); sq.advance_words(nmeas); }
        states
    };

    let gpu_screen = std::env::var("SKYWALK_GPU_SCREEN").is_ok_and(|v| v == "1");
    let census_sites = std::env::var("SKYWALK_CENSUS_META").ok()
        .map(|path| load_census_sites(&path, &ops));
    assert!(census_sites.is_none() || !gpu_screen, "census needs CPU state snapshots");
    let mut census_rows = census_sites.as_ref().map(|sites| vec![vec![[0u64; 4]; batches]; sites.len()]);
    let t = Instant::now();
    let refs = if gpu_screen {
        Vec::new()
    } else {
        let mut cpu_x = if let Some(nonces) = nonce_cfg.as_ref() {
            assert_eq!(nonces.len(), 1, "CPU nonce-reference packs support one nonce; use GPU screen for ranges");
            let mut x = fiat_reader(&ops, nonces[0]);
            let mut skipped = vec![0u8; OFFICIAL_SHOTS * 64];
            XofReader::read(&mut x, &mut skipped);
            x
        } else {
            let mut h = Shake256::default(); h.update(&measurement_tag); h.finalize_xof()
        };
        let mut sim = Simulator::new(nq as usize, nb as usize, &mut cpu_x);
        let mut refs = vec![[0u64; 4]; batches];
        for b in 0..batches {
            let bs = 64.min(n - b*64);
            let mask = if bs == 64 { u64::MAX } else { (1u64 << bs) - 1 };
            sim.clear_for_shot();
            for s in 0..bs {
                let (p,o,_) = ins[b*64+s];
                sim.set_register(&regs[0], p.0, s); sim.set_register(&regs[1], p.1, s);
                sim.set_register(&regs[2], o.0, s); sim.set_register(&regs[3], o.1, s);
            }
            let before = sim.stats.toffoli_gates;
            if let Some(sites) = census_sites.as_ref() {
                let mut start = 0usize;
                for (si, site) in sites.iter().enumerate() {
                    sim.apply_iter(ops[start..site.op].iter());
                    census_rows.as_mut().unwrap()[si][b] = census_masks(&sim, site, bs);
                    start = site.op;
                }
                sim.apply_iter(ops[start..].iter());
            } else {
                sim.apply_iter(ops.iter());
            }
            let mut bad = 0u64;
            for s in 0..bs {
                let e = ins[b*64+s].2;
                if sim.get_register(&regs[0],s) != e.0 || sim.get_register(&regs[1],s) != e.1 { bad |= 1u64 << s; }
            }
            let phase = sim.phase & mask;
            for r in &regs { for x in r { if let QubitOrBit::Qubit(q)=*x { *sim.qubit_mut(q)=0; } } }
            let mut anc = 0u64;
            for q in 0..nq { anc |= sim.qubit(QubitId(q)); }
            refs[b] = [bad, phase, anc & mask, sim.stats.toffoli_gates - before];
        }
        refs
    };
    let cpu_sim_s = t.elapsed().as_secs_f64();

    if let (Some(path), Some(sites), Some(rows)) = (
        std::env::var("SKYWALK_CENSUS_OUT").ok(), census_sites.as_ref(), census_rows.as_ref())
    {
        let mut cw = BufWriter::new(File::create(&path).unwrap());
        writeln!(cw, "phase,raw_tick,pair_site,emax,batch,valid_mask,escape55,escape56,escape57,escape58,value_failure,phase_failure,ancilla_failure").unwrap();
        for (si, site) in sites.iter().enumerate() {
            let mut totals = [0u64; 4];
            for b in 0..batches {
                let bs = 64.min(n - b * 64);
                let valid = if bs == 64 { u64::MAX } else { (1u64 << bs) - 1 };
                let m = rows[si][b];
                for j in 0..4 { totals[j] += (m[j] & valid).count_ones() as u64; }
                writeln!(cw, "{},{},{},{},{},{:016x},{:016x},{:016x},{:016x},{:016x},{:016x},{:016x},{:016x}",
                    site.phase, site.raw, site.site, site.emax, b, valid, m[0], m[1], m[2], m[3],
                    refs[b][0], refs[b][1], refs[b][2]).unwrap();
            }
            eprintln!("CENSUS raw={} pair_site={} phase={} emax={} escape55={} escape56={} escape57={} escape58={}",
                site.raw, site.site, site.phase, site.emax, totals[0], totals[1], totals[2], totals[3]);
        }
        cw.flush().unwrap();
        eprintln!("CENSUS_OUT {path}");
    }

    let exec_ops: Vec<&Op> = ops.iter().filter(|o| !matches!(o.kind, K::Register | K::AppendToRegister | K::DebugPrint)).collect();
    let mut w = BufWriter::new(File::create(out_path).unwrap());
    w.write_all(PACK_MAGIC).unwrap();
    let pack_version = if gpu_screen { 2u64 } else { 1u64 };
    for x in [pack_version, exec_ops.len() as u64, nq, nb, n as u64, batches as u64, nmeas as u64,
              out_regs.len() as u64, declared.len() as u64, max_depth as u64] { write_u64(&mut w, x); }
    for o in &exec_ops {
        w.write_all(&[o.kind as u8, 0]).unwrap();
        for x in [id16(o.q_control2.0), id16(o.q_control1.0), id16(o.q_target.0),
                  id16(o.c_target.0), id16(o.c_condition.0)] { write_u16(&mut w, x); }
        w.write_all(&[0u8; 4]).unwrap();
    }
    for x in &out_regs { write_u16(&mut w, *x); }
    for x in &declared { write_u16(&mut w, *x); }
    for x in &initial { write_u64(&mut w, *x); }
    for x in &expected { write_u64(&mut w, *x); }
    for s in &states { for x in s.st { write_u64(&mut w, x); } write_u64(&mut w, s.pt as u64); }
    for r in &refs { for x in r { write_u64(&mut w, *x); } }
    w.flush().unwrap();
    let total_s = all_start.elapsed().as_secs_f64();
    eprintln!("PACK ops={} exec={} nq={} nb={} shots={} batches={} rng_words_per_batch={} depth={}",
        ops.len(), exec_ops.len(), nq, nb, n, batches, nmeas, max_depth);
    if gpu_screen {
        eprintln!("CPU skipped (GPU screen pack v2; run a periodic v1 pack for full agreement)");
    } else {
        let val = refs.iter().map(|r| r[0].count_ones() as u64).sum::<u64>();
        let ph = refs.iter().map(|r| r[1].count_ones() as u64).sum::<u64>();
        let anc = refs.iter().map(|r| r[2].count_ones() as u64).sum::<u64>();
        let tof = refs.iter().map(|r| r[3]).sum::<u64>();
        eprintln!("CPU value={} phase_shots={} anc_shots={} T_total={} T_mean={:.3}", val, ph, anc, tof, tof as f64/n as f64);
    }
    eprintln!("TIMING load={load_s:.3}s input_prep={input_s:.3}s cpu_sim={cpu_sim_s:.3}s total={total_s:.3}s");
}
