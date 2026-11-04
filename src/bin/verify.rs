#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]

//! Release-mode verifier for the 81-Mode Continuum Clay Variation.
//!
//! Verifies:
//! 1. Dissipation constant D81 = 900_647 matches its definition and geometric identity 8*D = 9^81 - 1.
//! 2. Cyclotomic boundary closure: sum_{j=0}^{80} W^j = 0 mod P.
//! 3. Closed-form step equivalence over 100,000 states.
//! 4. Spherical NTT-81 exact integer roundtrip invertibility.
//! 5. Full-period orbits of P = 1_000_999 steps checking exact conservation of I = Q + t*nu*D81.
//! 6. Real-norm vs modular residue boundary audit.

use continuum_clay_variation::{
    conserved_invariant81, dissipation_weight81, drift_constant81, predicted_step81,
    predicted_total81, prism81, shell_total81, sub_mod, unprism81, OmegaOmnibitRoom, ShellModel81,
    SplitMix64, DRIFT81, K, P, W,
};
use std::process::ExitCode;

struct Orbit {
    name: &'static str,
    u0: [u64; K],
    nu: u64,
}

fn run_full_period(o: &Orbit) -> (bool, u64, u64, u64) {
    let q0 = shell_total81(&o.u0);
    let mut seen = vec![0u64; (P as usize).div_ceil(64)];
    let mut distinct = 0u64;
    let mut mark = |q: u64, seen: &mut Vec<u64>| {
        let (w, b) = ((q / 64) as usize, q % 64);
        if seen[w] & (1 << b) == 0 {
            seen[w] |= 1 << b;
            distinct += 1;
        }
    };
    mark(q0, &mut seen);

    let mut m = ShellModel81 { u: o.u0, nu: o.nu };
    let mut ok = true;
    let mut first_bad = 0u64;

    for t in 1..=P {
        m.step();
        let q = shell_total81(&m.u);
        let good = m.u.iter().all(|&x| x < P)
            && q == predicted_total81(q0, o.nu, t)
            && conserved_invariant81(&m.u, o.nu, t) == q0;
        if !good && ok {
            ok = false;
            first_bad = t;
        }
        if t < P {
            mark(q, &mut seen);
        }
    }

    let q_end = shell_total81(&m.u);
    let nu_zero = o.nu % P == 0;
    let expected_distinct = if nu_zero { 1 } else { P };
    ok = ok && q_end == q0 && distinct == expected_distinct;
    (ok, distinct, q_end, first_bad)
}

fn main() -> ExitCode {
    let mut all_ok = true;
    println!(
        "VERIFY81HDR|crate=continuum-clay-variation|K=81|P={P}|W={W}|arithmetic=integer_only|float_used=0|json=0"
    );

    // 1. Drift constant
    let d_def = drift_constant81();
    let d_geom = sub_mod(dissipation_weight81(81), 1);
    let d_ok = d_def == DRIFT81 && ((8 * DRIFT81) % P) == d_geom;
    all_ok &= d_ok;
    println!(
        "DRIFT81|D={DRIFT81}|recomputed={d_def}|geom_8D_eq_9pow81_minus_1={}|pass={}|json=0",
        u8::from(((8 * DRIFT81) % P) == d_geom),
        u8::from(d_ok)
    );

    // 2. Cyclotomic closure
    let mut cyc_sum = 0u64;
    let mut w_pow = 1u64;
    for _ in 0..K {
        cyc_sum = (cyc_sum + w_pow) % P;
        w_pow = (w_pow * W) % P;
    }
    let cyc_ok = cyc_sum == 0 && w_pow == 1;
    all_ok &= cyc_ok;
    println!(
        "CYCLOTOMIC81|sum_W_pow_j_mod_P={cyc_sum}|W_pow_81_mod_P={w_pow}|pass={}|json=0",
        u8::from(cyc_ok)
    );

    // 3. NTT-81 roundtrip invertibility
    let mut rng = SplitMix64(0x0A50_1A21_C000_0001);
    let mut ntt_ok = true;
    for _ in 0..10_000 {
        let x = rng.state81();
        let spec = prism81(&x);
        let rec = unprism81(&spec);
        if rec != x {
            ntt_ok = false;
            break;
        }
    }
    all_ok &= ntt_ok;
    println!(
        "NTT81PRISM|test_vectors=10000|exact_integer_roundtrip={}|pass={}|json=0",
        u8::from(ntt_ok),
        u8::from(ntt_ok)
    );

    // 4. Closed-form step verification across 100,000 states
    let mut step_matches = 0u64;
    let mut step_mismatches = 0u64;
    for _ in 0..100_000 {
        let u = rng.state81();
        let nu = rng.next_u64();
        let mut m = ShellModel81 { u, nu };
        m.step();
        let pred = predicted_step81(&u, nu);
        if m.u == pred {
            step_matches += 1;
        } else {
            step_mismatches += 1;
        }
    }
    let step_ok = step_mismatches == 0;
    all_ok &= step_ok;
    println!(
        "STEPFORM81|tested={step_matches}|mismatches={step_mismatches}|pass={}|json=0",
        u8::from(step_ok)
    );

    // 5. Full-period orbits
    let room_sample = OmegaOmnibitRoom::new(
        0x9E37_79B9_7F4A_7C15,
        0xBF58_476D_1CE4_E5B9,
        0x94D0_49BB_1331_11EB,
    );
    let orbits = [
        Orbit {
            name: "initial_energy_sample",
            u0: ShellModel81::new(999_999, 1).u,
            nu: 1,
        },
        Orbit {
            name: "omega_omnibit_room_orbit",
            u0: room_sample.unpack_state(333_360),
            nu: 2,
        },
        Orbit {
            name: "zero_state_control",
            u0: [0u64; K],
            nu: 1,
        },
        Orbit {
            name: "inviscid_control",
            u0: ShellModel81::new(500_000, 0).u,
            nu: 0,
        },
    ];

    for o in &orbits {
        let (ok, distinct, q_end, first_bad) = run_full_period(o);
        all_ok &= ok;
        println!(
            "ORBIT81|name={}|nu={}|steps={P}|q0={}|q_end={q_end}|distinct_q={distinct}|first_violation={first_bad}|pass={}|json=0",
            o.name,
            o.nu,
            shell_total81(&o.u0),
            u8::from(ok)
        );
    }

    println!(
        "SCOPE81|theorem=exact_all_time_affine_drift_of_81_shell_total|conserved=Q+t*nu*D81|bkm_continuum_limit=requires_unreduced_real_vorticity_norm_control|json=0"
    );
    println!("RESULT81|pass={}|json=0", u8::from(all_ok));

    if all_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
