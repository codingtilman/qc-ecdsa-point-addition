use alloy_primitives::U256;
use crate::circuit::{analyze_ops, BitId, Op, OperationType, QubitId, QubitOrBit, RegisterId};
use crate::sim::Simulator;
use crate::weierstrass_elliptic_curve::WeierstrassEllipticCurve;
use sha3::{digest::{ExtendableOutput, Update, XofReader}, Shake256};
use std::fs::File;
use std::io::{BufReader, Read};

const MAGIC: &[u8; 8] = b"QECCOPSZ";
const OP_BYTES: usize = 56;

fn kind(v: u32) -> OperationType {
    match v {
        0 => OperationType::Neg, 1 => OperationType::Register, 2 => OperationType::AppendToRegister,
        3 => OperationType::BitInvert, 4 => OperationType::BitStore0, 5 => OperationType::BitStore1,
        6 => OperationType::X, 7 => OperationType::Z, 8 => OperationType::CX, 9 => OperationType::CZ,
        10 => OperationType::Swap, 11 => OperationType::R, 12 => OperationType::Hmr,
        13 => OperationType::CCX, 14 => OperationType::CCZ, 15 => OperationType::PushCondition,
        16 => OperationType::PopCondition, 17 => OperationType::DebugPrint,
        _ => panic!("bad op kind {v}"),
    }
}
fn u(rec: &[u8], off: usize) -> u64 { u64::from_le_bytes(rec[off..off + 8].try_into().unwrap()) }
fn load(path: &str) -> Vec<Op> {
    let mut file = File::open(path).unwrap();
    let mut header = [0u8; 16]; file.read_exact(&mut header).unwrap();
    assert_eq!(&header[..8], MAGIC);
    let n = u64::from_le_bytes(header[8..].try_into().unwrap()) as usize;
    let mut dec = zstd::stream::read::Decoder::new(BufReader::new(file)).unwrap();
    dec.window_log_max(27).unwrap();
    let mut out = Vec::with_capacity(n);
    let mut rec = [0u8; OP_BYTES];
    for _ in 0..n {
        dec.read_exact(&mut rec).unwrap();
        let op = Op { kind: kind(u32::from_le_bytes(rec[0..4].try_into().unwrap())),
            q_control2: QubitId(u(&rec, 8)), q_control1: QubitId(u(&rec, 16)), q_target: QubitId(u(&rec, 24)),
            c_target: BitId(u(&rec, 32)), c_condition: BitId(u(&rec, 40)), r_target: RegisterId(u(&rec, 48)) };
        op.validate(); out.push(op);
    }
    out
}
fn curve() -> WeierstrassEllipticCurve {
    let h = |s: &str| U256::from_str_radix(s, 16).unwrap();
    WeierstrassEllipticCurve { modulus: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F"),
        a: U256::ZERO, b: U256::from(7),
        gx: h("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798"),
        gy: h("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8"),
        order: h("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141") }
}
type Input = ((U256,U256),(U256,U256),(U256,U256));
fn inputs(n: usize) -> Vec<Input> {
    let c = curve();
    let mut h = Shake256::default(); h.update(b"ecdsafail-independent-paired-v1");
    let mut x = h.finalize_xof(); let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let mut a = [0u8;32]; let mut b = [0u8;32]; XofReader::read(&mut x,&mut a); XofReader::read(&mut x,&mut b);
        let p = c.mul(c.gx,c.gy,U256::from_le_bytes(a)); let q = c.mul(c.gx,c.gy,U256::from_le_bytes(b));
        if p.0 == q.0 || (p.0.is_zero()&&p.1.is_zero()) || (q.0.is_zero()&&q.1.is_zero()) { continue; }
        out.push((p,q,c.add(p.0,p.1,q.0,q.1)));
    }
    out
}
struct Report { q: usize, t: f64, val: Vec<bool>, phase: Vec<bool>, anc: Vec<bool> }
fn run(ops: &[Op], ins: &[Input]) -> Report {
    let (nq,nb,_,regs)=analyze_ops(ops.iter()); assert_eq!(regs.len(),4);
    let mut h=Shake256::default(); h.update(b"ecdsafail-independent-paired-measurements-v1");
    let mut x=h.finalize_xof(); let mut sim=Simulator::new(nq as usize,nb as usize,&mut x);
    let mut val=vec![false;ins.len()]; let mut phase=val.clone(); let mut anc=val.clone();
    for base in (0..ins.len()).step_by(64) {
        let bs=64.min(ins.len()-base); let mask=if bs==64 {u64::MAX} else {(1u64<<bs)-1}; sim.clear_for_shot();
        for s in 0..bs { let (p,q,_)=ins[base+s]; sim.set_register(&regs[0],p.0,s); sim.set_register(&regs[1],p.1,s); sim.set_register(&regs[2],q.0,s); sim.set_register(&regs[3],q.1,s); }
        sim.apply_iter(ops.iter());
        for s in 0..bs { let e=ins[base+s].2; val[base+s]=sim.get_register(&regs[0],s)!=e.0||sim.get_register(&regs[1],s)!=e.1; phase[base+s]=(sim.phase>>s)&1!=0; }
        for r in &regs { for qb in r { if let QubitOrBit::Qubit(q)=*qb {*sim.qubit_mut(q)=0;} } }
        let mut dirty=0u64; for q in 0..nq {dirty|=sim.qubit(QubitId(q))&mask;} for s in 0..bs {anc[base+s]=(dirty>>s)&1!=0;}
    }
    Report { q:nq as usize, t:sim.stats.toffoli_gates as f64/ins.len() as f64, val, phase, anc }
}
fn count(v:&[bool])->usize {v.iter().filter(|&&x|x).count()}
fn discord(a:&[bool],b:&[bool])->usize {a.iter().zip(b).filter(|(x,y)|x!=y).count()}
fn union3(a:&[bool],b:&[bool],c:&[bool])->Vec<bool> {a.iter().zip(b).zip(c).map(|((&x,&y),&z)|x||y||z).collect()}
fn only(a:&[bool],b:&[bool])->(usize,usize) {let mut ao=0;let mut bo=0;for (&x,&y) in a.iter().zip(b){if x&&!y{ao+=1}if y&&!x{bo+=1}}(ao,bo)}

#[test]
fn paired_ops_10000() {
    let n:usize=std::env::var("PAIRED_N").ok().and_then(|v|v.parse().ok()).unwrap_or(10_000);
    let base=load("/workspace/project/ecdsafail-challenge/ops.bin");
    let cand=load("/workspace/project/ecdsa-research/baseline/ops.bin");
    let ins=inputs(n);
    let a=run(&base,&ins); let b=run(&cand,&ins);
    eprintln!("PAIRED n={n} baseline Q={} T={:.3} value={} phase={} anc={}",a.q,a.t,count(&a.val),count(&a.phase),count(&a.anc));
    eprintln!("PAIRED n={n} candidate Q={} T={:.3} value={} phase={} anc={}",b.q,b.t,count(&b.val),count(&b.phase),count(&b.anc));
    eprintln!("PAIRED n={n} discord value={} phase={} anc={}",discord(&a.val,&b.val),discord(&a.phase,&b.phase),discord(&a.anc,&b.anc));
    let ab=union3(&a.val,&a.phase,&a.anc);let bb=union3(&b.val,&b.phase,&b.anc);
    eprintln!("PAIRED n={n} any baseline={} candidate={} baseline_only={} candidate_only={}",count(&ab),count(&bb),only(&ab,&bb).0,only(&ab,&bb).1);
    eprintln!("PAIRED n={n} only value={:?} phase={:?} anc={:?}",only(&a.val,&b.val),only(&a.phase,&b.phase),only(&a.anc,&b.anc));
}
