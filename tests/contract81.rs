#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]

//! Mathematical contract tests for the 81-Mode Continuum Clay Variation.

use continuum_clay_variation::{
    conserved_invariant81, dissipation_weight81, drift_constant81, predicted_step81,
    predicted_total81, prism81, shell_total81, sub_mod, unprism81, OmegaOmnibitRoom, ShellModel81,
    SplitMix64, DRIFT81, K, P, W, W_INV,
};

fn sum_mod(x: &[u64; K]) -> u64 {
    x.iter().fold(0u64, |acc, &val| (acc + val % P) % P)
}

#[test]
fn test_drift_constant_and_geometric_identity() {
    assert_eq!(drift_constant81(), DRIFT81);
    assert_eq!(DRIFT81, 900_647);
    assert_ne!(DRIFT81, 0);

    // 8 * D81 = 9^81 - 1 mod P
    let nine_81 = dissipation_weight81(81);
    assert_eq!((8 * DRIFT81) % P, sub_mod(nine_81, 1));
}

#[test]
fn test_cyclotomic_boundary_closure_and_root_order() {
    assert_eq!(continuum_clay_variation::pow_mod(W, 81), 1);
    assert_ne!(continuum_clay_variation::pow_mod(W, 27), 1);
    assert_eq!((W * W_INV) % P, 1);

    let mut cyc = 0u64;
    let mut term = 1u64;
    for _ in 0..K {
        cyc = (cyc + term) % P;
        term = (term * W) % P;
    }
    assert_eq!(
        cyc, 0,
        "Law 20 spherical boundary closure must hold: sum W^j = 0 mod P"
    );
}

#[test]
fn test_81_point_spherical_ntt_roundtrip_is_exact() {
    let mut rng = SplitMix64(0x0A50_1A21_D000_0001);
    for _ in 0..1_000 {
        let x = rng.state81();
        let spectrum = prism81(&x);
        let back = unprism81(&spectrum);
        assert_eq!(
            x, back,
            "NTT-81 spherical prism round-trip must be bit-exact"
        );
    }
}

#[test]
fn test_triad_non_linear_flux_sums_identically_to_zero_across_states() {
    let mut rng = SplitMix64(0x0A50_1A21_D000_0002);

    // Extreme states
    let mut states = vec![[0u64; K], [1u64; K], [P - 1; K]];
    for i in 0..K {
        let mut u = [0u64; K];
        u[i] = P / 2;
        states.push(u);
    }

    // Random states
    for _ in 0..5_000 {
        states.push(rng.state81());
    }

    // Omega-Omnibit rooms (n x 3 x 3 x 3)
    for _ in 0..1_000 {
        let room = rng.room();
        states.push(room.unpack_state(333_360));
    }

    for u in &states {
        let model = ShellModel81 { u: *u, nu: 1 };
        let nl = model.non_linear_transfer();
        assert_eq!(
            sum_mod(&nl),
            0,
            "Non-linear convective transfer must sum to 0 mod P"
        );

        let spec = prism81(&nl);
        assert_eq!(spec[0], 0, "Law 0 DC center X[0] must be strictly 0 mod P");
    }
}

#[test]
fn test_step_closed_form_and_affine_drift() {
    let mut rng = SplitMix64(0x0A50_1A21_D000_0003);
    for _ in 0..3_000 {
        let u = rng.state81();
        let nu = rng.next_u64();
        let mut model = ShellModel81 { u, nu };
        let q0 = shell_total81(&u);

        model.step();
        let pred = predicted_step81(&u, nu);
        assert_eq!(model.u, pred, "Step must match closed-form expression");

        let q1 = shell_total81(&model.u);
        assert_eq!(q1, predicted_total81(q0, nu, 1));
        assert_eq!(conserved_invariant81(&model.u, nu, 1), q0);
    }
}

#[test]
fn test_inviscid_flow_conserves_q_for_thousands_of_steps() {
    let mut rng = SplitMix64(0x0A50_1A21_D000_0004);
    for _ in 0..10 {
        let u = rng.state81();
        let q0 = shell_total81(&u);
        let mut model = ShellModel81 { u, nu: 0 };

        for _ in 0..1_000 {
            model.step();
            assert_eq!(shell_total81(&model.u), q0);
        }
    }
}

#[test]
fn test_omega_omnibit_room_serialization_and_invariants() {
    let room = OmegaOmnibitRoom::new(
        0x1234_5678_9ABC_DEF0,
        0xFEDC_BA98_7654_3210,
        0xA5A5_5A5A_0F0F_F0F0,
    );
    let bytes = room.to_bytes();
    assert_eq!(bytes.len(), 24);

    let decoded = OmegaOmnibitRoom::from_bytes(&bytes);
    assert_eq!(room, decoded);

    let u = room.unpack_state(500_000);
    assert_eq!(u.len(), 81);

    let mut model = ShellModel81 { u, nu: 2 };
    let nl = model.non_linear_transfer();
    assert_eq!(sum_mod(&nl), 0);
    assert_eq!(prism81(&nl)[0], 0);

    let q0 = shell_total81(&model.u);
    for t in 1..=100 {
        model.step();
        assert_eq!(conserved_invariant81(&model.u, 2, t), q0);
    }
}

// ---------------------------------------------------------------- Scope & Boundary Audits

#[test]
fn test_scope_positive_viscosity_moves_zero_state() {
    let mut model = ShellModel81 {
        u: [0u64; K],
        nu: 1,
    };
    model.step();
    assert_eq!(model.u[0], P - 1);
    assert!(model.u.iter().all(|&x| x != 0));
    assert_eq!(shell_total81(&model.u), sub_mod(0, DRIFT81));
}

#[test]
fn test_scope_linear_conservation_does_not_imply_quadratic_energy_cancellation() {
    let mut u = [0u64; K];
    u[0] = 1;
    u[1] = 1;
    let model = ShellModel81 { u, nu: 0 };
    let nl = model.non_linear_transfer();
    assert_eq!(sum_mod(&nl), 0);

    let dot = u
        .iter()
        .zip(nl.iter())
        .fold(0u64, |acc, (&x, &y)| (acc + (x * y) % P) % P);
    assert_ne!(
        dot, 0,
        "sum u_n * NL_n is not zero for linear balance transfer"
    );
}

#[test]
fn test_scope_modular_enstrophy_vs_unreduced_real_energy() {
    let mut model = ShellModel81::new(999_999, 1);
    for _ in 0..50 {
        model.step();
        // Modular residue check: always strictly < P
        assert!(model.total_enstrophy() < P);
        // Unreduced integer energy check:
        let real_e = model.unreduced_energy_u128();
        assert!(real_e > 0);
    }
}
