//! Exact observable conversion from 24^3 Taylor–Green D3Q27 occupations
//! into 81 finite-field Clay registers: 3x3x3 blocks x 3 momentum axes.
//! The conversion is exact for these 81 observables, not for the full state.
//! Three time steps are independently replayed from pinned Taylor–Green
//! input, against the actual Clay crate and sealed Taylor–Green measurements.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::float_cmp, clippy::cast_precision_loss)]

use continuum_clay_variation::{prism81, unprism81, ShellModel81, K, P};
use std::{env, fs, process::ExitCode};

const L: usize = 24;
const CHANNELS: usize = 27;
const CELLS: usize = L * L * L;
const BLOCK_SIDE: usize = 8;
const BLOCKS_PER_AXIS: usize = 3;
const STEPS: usize = 3;
const ABS_COMPONENT_LIMIT: i64 = (BLOCK_SIDE * BLOCK_SIDE * BLOCK_SIDE * 18) as i64;
const TG_EXPECTED: [[i64; 4]; 4] = [
    [746_496, 106_496, 108_032, 35_328],
    [746_496, 382_752, 1_225_664, 37_128],
    [746_496, 537_904, 1_876_856, 34_600],
    [746_496, 573_144, 2_038_608, 33_688],
];

#[derive(Clone)]
struct Gate {
    q: [usize; 4],
    lut: [u8; 81],
}

#[derive(Clone)]
struct Obs {
    u: [u64; K],
    signed: [i64; K],
    global: [i64; 3],
    mass: i64,
    e2: i64,
    kj: i64,
    wj: i64,
    amplitude: i64,
}

fn velocities() -> [[i64; 4]; CHANNELS] {
    let mut result = [[0i64; 4]; CHANNELS];
    let mut index = 0;
    for x in -1i64..=1 {
        for y in -1i64..=1 {
            for z in -1i64..=1 {
                result[index] = [x, y, z, x * x + y * y + z * z];
                index += 1;
            }
        }
    }
    result
}

fn decode(mut code: usize) -> [u8; 4] {
    let mut result = [0u8; 4];
    for digit in &mut result {
        *digit = (code % 3) as u8;
        code /= 3;
    }
    result
}
fn encode(d: [u8; 4]) -> usize {
    usize::from(d[0])
        + 3 * usize::from(d[1])
        + 9 * usize::from(d[2])
        + 27 * usize::from(d[3])
}
fn invariant(d: [u8; 4], q: [usize; 4], v: &[[i64; 4]; CHANNELS]) -> [i64; 5] {
    let mut a = [0; 5];
    for k in 0..4 {
        a[0] += i64::from(d[k]);
        for axis in 0..4 {
            a[axis + 1] += i64::from(d[k]) * v[q[k]][axis];
        }
    }
    a
}
fn checked_gates(bytes: &[u8], v: &[[i64; 4]; CHANNELS]) -> Result<Vec<Gate>, String> {
    if bytes.len() != 288 * 85 {
        return Err(format!("expected 288 serialized gates; got {} bytes", bytes.len()));
    }
    let mut out = Vec::with_capacity(288);
    for (i, data) in bytes.chunks_exact(85).enumerate() {
        let q = [
            usize::from(data[0]),
            usize::from(data[1]),
            usize::from(data[2]),
            usize::from(data[3]),
        ];
        if q.iter().any(|&x| x >= CHANNELS) {
            return Err(format!("out-of-range gate channel at {i}"));
        }
        for a in 0..4 {
            for b in a + 1..4 {
                if q[a] == q[b] {
                    return Err(format!("duplicate channel in gate {i}"));
                }
            }
        }
        let mut lut = [0u8; 81];
        lut.copy_from_slice(&data[4..]);
        if lut[40] != 40 {
            return Err(format!("null equilibrium not fixed at gate {i}"));
        }
        for (mask, &mapped) in lut.iter().enumerate() {
            let m = usize::from(mapped);
            if m >= 81 || lut[m] as usize != mask {
                return Err(format!("not an involution at gate {i}, code {mask}"));
            }
            if invariant(decode(mask), q, v) != invariant(decode(m), q, v) {
                return Err(format!("gate {i} violates exact local conservation"));
            }
        }
        out.push(Gate { q, lut });
    }
    Ok(out)
}

fn checked_trits(state: &[u8]) -> Result<(), String> {
    if state.len() != CELLS * CHANNELS {
        return Err(format!("trit count {} != {}", state.len(), CELLS * CHANNELS));
    }
    if state.iter().any(|&n| n > 2) {
        return Err("non-trit channel occupancy".into());
    }
    Ok(())
}

fn step_tg(s: &mut Vec<u8>, t: usize, gates: &[Gate], v: &[[i64; 4]; CHANNELS]) {
    // Reproduce the original 2026-10-30 Taylor-Green gate scheduler exactly.
    for cell in 0..CELLS {
        let base = cell * CHANNELS;
        for j in 0..32 {
            let g = &gates[(t * 32 + cell * 17 + j * 37) % gates.len()];
            let mask = encode([
                s[base + g.q[0]],
                s[base + g.q[1]],
                s[base + g.q[2]],
                s[base + g.q[3]],
            ]);
            let next = decode(g.lut[mask] as usize);
            for (k, &q) in g.q.iter().enumerate() {
                s[base + q] = next[k];
            }
        }
    }
    let mut next = vec![0u8; CELLS * CHANNELS];
    for x in 0..L {
        for y in 0..L {
            for z in 0..L {
                let base = ((x * L + y) * L + z) * CHANNELS;
                for (k, dir) in v.iter().enumerate() {
                    let nx = (x as i64 + dir[0]).rem_euclid(L as i64) as usize;
                    let ny = (y as i64 + dir[1]).rem_euclid(L as i64) as usize;
                    let nz = (z as i64 + dir[2]).rem_euclid(L as i64) as usize;
                    next[((nx * L + ny) * L + nz) * CHANNELS + k] = s[base + k];
                }
            }
        }
    }
    *s = next;
}

fn scramble(s: &[u8]) -> Vec<u8> {
    let mut perm: Vec<usize> = (0..CELLS).collect();
    let mut x: u64 = 20_261_030;
    for i in (1..CELLS).rev() {
        x = (1_103_515_245u64 * x + 12_345) % 2_147_483_648;
        perm.swap(i, (x % (i as u64 + 1)) as usize);
    }
    let mut out = vec![0u8; s.len()];
    for (i, &p) in perm.iter().enumerate() {
        out[i * CHANNELS..(i + 1) * CHANNELS]
            .copy_from_slice(&s[p * CHANNELS..(p + 1) * CHANNELS]);
    }
    out
}
fn cell_index(x: usize, y: usize, z: usize) -> usize {
    ((x % L) * L + (y % L)) * L + z % L
}
fn sin_sign(x: usize) -> i64 {
    if x < 12 { 1 } else { -1 }
}
fn cos_sign(x: usize) -> i64 {
    if !(6..18).contains(&x) { 1 } else { -1 }
}
fn project(s: &[u8], v: &[[i64; 4]; CHANNELS]) -> Result<Obs, String> {
    checked_trits(s)?;
    let mut signed = [0i64; K];
    let mut global = [0i64; 3];
    let mut mass = 0i64;
    let mut e2 = 0i64;
    let mut kj = 0i64;
    let mut j = vec![[0i64; 3]; CELLS];
    for x in 0..L {
        for y in 0..L {
            for z in 0..L {
                let cell = cell_index(x, y, z);
                let block = (x / BLOCK_SIDE) * 9
                    + (y / BLOCK_SIDE) * 3 + z / BLOCK_SIDE;
                for (k, dir) in v.iter().enumerate() {
                    let n = i64::from(s[cell * CHANNELS + k]);
                    mass += n;
                    e2 += n * dir[3];
                    for axis in 0..3 {
                        let p = n * dir[axis];
                        j[cell][axis] += p;
                        signed[block * 3 + axis] += p;
                        global[axis] += p;
                    }
                }
                kj += j[cell].iter().map(|p| p * p).sum::<i64>();
            }
        }
    }
    if signed.iter().any(|x| x.abs() > ABS_COMPONENT_LIMIT) {
        return Err("block momentum exceeds proved finite-grid magnitude bound".into());
    }
    let mut wj = 0i64;
    let mut amplitude = 0i64;
    for x in 0..L {
        for y in 0..L {
            for z in 0..L {
                let q = |a: i64, b: i64, c: i64| -> usize {
                    cell_index(
                        a.rem_euclid(L as i64) as usize,
                        b.rem_euclid(L as i64) as usize,
                        c.rem_euclid(L as i64) as usize,
                    )
                };
                let (x, y, z) = (x as i64, y as i64, z as i64);
                let wx = (j[q(x,y+1,z)][2] - j[q(x,y-1,z)][2])
                    - (j[q(x,y,z+1)][1] - j[q(x,y,z-1)][1]);
                let wy = (j[q(x,y,z+1)][0] - j[q(x,y,z-1)][0])
                    - (j[q(x+1,y,z)][2] - j[q(x-1,y,z)][2]);
                let wz = (j[q(x+1,y,z)][1] - j[q(x-1,y,z)][1])
                    - (j[q(x,y+1,z)][0] - j[q(x,y-1,z)][0]);
                wj += wx * wx + wy * wy + wz * wz;
                let c = q(x,y,z);
                let (x,y,z) = (x as usize,y as usize,z as usize);
                amplitude += j[c][0] * sin_sign(x) * cos_sign(y) * cos_sign(z)
                    - j[c][1] * cos_sign(x) * sin_sign(y) * cos_sign(z);
            }
        }
    }
    let mut u = [0u64; K];
    for i in 0..K {
        u[i] = signed[i].rem_euclid(P as i64) as u64;
        let recovered = if u[i] > P / 2 {
            u[i] as i64 - P as i64
        } else {
            u[i] as i64
        };
        if recovered != signed[i] {
            return Err(format!("centered block momentum not recoverable at {i}"));
        }
    }
    if unprism81(&prism81(&u)) != u {
        return Err("Clay's exact 81-point transform did not invert".into());
    }
    if u.iter().fold(0, |a, &v| (a + v) % P)
        != global.iter().sum::<i64>().rem_euclid(P as i64) as u64
    {
        return Err("81-register sum does not equal global momentum sum".into());
    }
    Ok(Obs { u, signed, global, mass, e2, kj, wj, amplitude })
}
fn csv_values<T: std::fmt::Display>(items: &[T]) -> String {
    items.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}
fn q(u: &[u64; K]) -> u64 {
    u.iter().fold(0u64, |a, &x| (a + x) % P)
}

fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 3 {
        return Err("required args: INIT_TGV.trits INIT_NULL.trits GATES.lut".into());
    }
    if K != BLOCKS_PER_AXIS.pow(3) * 3 {
        return Err("incorrect block momentum layout".into());
    }
    let v = velocities();
    let tg = fs::read(&args[0]).map_err(|e| e.to_string())?;
    let null = fs::read(&args[1]).map_err(|e| e.to_string())?;
    let gate_bytes = fs::read(&args[2]).map_err(|e| e.to_string())?;
    checked_trits(&tg)?;
    checked_trits(&null)?;
    if null.iter().any(|&n| n != 1) {
        return Err("NULL pinned input is not uniform equilibrium".into());
    }
    let gates = checked_gates(&gate_bytes, &v)?;
    let mut states = [tg.clone(), scramble(&tg), null.clone()];
    let labels = ["TGV", "SCR", "NULL"];
    let mut previous = [
        project(&states[0], &v)?,
        project(&states[1], &v)?,
        project(&states[2], &v)?,
    ];
    if [previous[0].e2, previous[0].kj, previous[0].wj, previous[0].amplitude]
        != TG_EXPECTED[0]
    {
        return Err("sealed TGV tick-zero metrics differ".into());
    }
    let initial_global = [previous[0].global, previous[1].global, previous[2].global];
    let initial_mass = [previous[0].mass, previous[1].mass, previous[2].mass];
    let initial_e2 = [previous[0].e2, previous[1].e2, previous[2].e2];
    if initial_global[0] != initial_global[1] {
        return Err("shuffling changed global momentum".into());
    }
    let mut seq_clay = [
        previous[0].u, previous[1].u, previous[2].u
    ];
    let rest = 13; // x=0,y=0,z=0 is channel 13 in the ordered D3Q27 array.
    if v[rest] != [0,0,0,0] {
        return Err("rest channel index moved".into());
    }
    let mut one_rest = vec![0u8; CELLS * CHANNELS];
    one_rest[rest] = 1;
    if project(&vec![0u8; CELLS * CHANNELS], &v)?.u
        != project(&one_rest, &v)?.u
        || project(&one_rest, &v)?.mass != 1
    {
        return Err("non-injectivity control failed".into());
    }
    let mut forced_null_clay = ShellModel81 {u:[0;K],nu:1};
    forced_null_clay.step();
    if forced_null_clay.u.iter().all(|&x|x==0) {
        return Err("forced Clay zero moved unexpectedly little".into());
    }
    println!("BRIDGE_HEADER|model=TG24_D3Q27_to_actual_Clay81|steps=3|arms=TGV,SCR,NULL|blocks=3x3x3|registers=81|gates={}|P={P}|source=2026-10-30-noise-test|json=0", gates.len());
    for t in 1..=STEPS {
        for (i, label) in labels.iter().enumerate() {
            let old_state = states[i].clone();
            step_tg(&mut states[i], t - 1, &gates, &v);
            let current = project(&states[i], &v)?;
            if current.global != initial_global[i]
                || current.mass != initial_mass[i]
                || current.e2 != initial_e2[i]
            {
                return Err(format!("conservation error arm={label} t={t}"));
            }
            if *label == "TGV"
                && [current.e2, current.kj, current.wj, current.amplitude]
                    != TG_EXPECTED[t]
            {
                return Err(format!("pinned TG result mismatch at t={t}"));
            }
            if *label == "NULL"
                && (states[i] != null || current.u.iter().any(|&v|v!=0))
            {
                return Err(format!("null equilibrium moved at t={t}"));
            }
            let mut local_clay = ShellModel81 {u:previous[i].u, nu:0};
            local_clay.step();
            let mut trajectory_clay = ShellModel81 {u:seq_clay[i], nu:0};
            trajectory_clay.step();
            seq_clay[i] = trajectory_clay.u;
            if q(&local_clay.u) != q(&previous[i].u)
                || q(&seq_clay[i]) != q(&previous[i].u)
                || q(&current.u) != q(&previous[i].u)
            {
                return Err(format!("momentum sum not preserved arm={label} t={t}"));
            }
            let local_defect = (0..K).filter(|&k| current.u[k] != local_clay.u[k]).count();
            let seq_defect = (0..K).filter(|&k| current.u[k] != seq_clay[i][k]).count();
            let moved_trits = old_state.iter().zip(states[i].iter())
                .filter(|(a,b)| a!=b).count();
            let moved_registers = (0..K).filter(|&k| current.u[k]!=previous[i].u[k]).count();
            if *label=="TGV" && local_defect==0 {
                return Err(format!("unpredicted commutation at t={t}: must report actual outcome"));
            }
            if *label=="NULL" && (local_defect != 0 || moved_trits != 0) {
                return Err("null comparator was not stationary".into());
            }
            println!(
                "LEAF|t={t}|arm={label}|mass={}|E2={}|KJ={}|WJ={}|A={}|global={}|Q={}|moved_trits={moved_trits}|moved_registers={moved_registers}|local_defect={local_defect}|seq_defect={seq_defect}|signed81={}|u81={}|json=0",
                current.mass, current.e2,current.kj,current.wj,current.amplitude,
                csv_values(&current.global), q(&current.u),
                csv_values(&current.signed),csv_values(&current.u)
            );
            previous[i] = current;
        }
        println!("STEP_RESULT|t={t}|leaves=3|status=PASS|json=0");
    }
    println!("BRIDGE_RESULT|steps=3|leaves=3|arms=3|signed_observable_conversion=EXACT|NTT=PASS|global_momentum=PASS|tg_replay_t0_t3=PASS|full_state_inverse=REFUTED|dynamical_conjugacy=REFUTED_FOR_THIS_MAPPING|PDE_bridge=OPEN|status=PASS|json=0");
    Ok(())
}
fn main() -> ExitCode {
    match run(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("BRIDGE_FAIL|reason={}|status=FAIL|json=0", e.replace(['\n','|'], " "));
            ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn three_axes_give_81_registers() {
        assert_eq!(BLOCKS_PER_AXIS.pow(3)*3,K);
        assert_eq!(ABS_COMPONENT_LIMIT,9216);
        assert!(ABS_COMPONENT_LIMIT < (P/2) as i64);
    }
    #[test]
    fn invalid_data_fails_closed() {
        assert!(checked_trits(&[0; 3]).is_err());
        let mut s=vec![1u8;CELLS*CHANNELS];
        s[0]=3;
        assert!(checked_trits(&s).is_err());
        assert!(checked_gates(&vec![0u8; 288*85],&velocities()).is_err());
    }
    #[test]
    fn rest_occupation_is_not_captured_by_momentum() {
        let v=velocities();
        assert_eq!(v[13],[0,0,0,0]);
        let s=vec![0u8; CELLS*CHANNELS];
        let mut t=s.clone();
        t[13]=2;
        let a=project(&s,&v).unwrap();
        let b=project(&t,&v).unwrap();
        assert_eq!(a.u,b.u);
        assert_ne!(a.mass,b.mass);
    }
}
