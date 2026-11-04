//! Independent integer oracles for the frozen 81-mode dynamics.
//!
//! These tests deliberately do not use production arithmetic, nonlinear transfer,
//! predicted-step, or transform routines when obtaining the expected result.
//! Existing 243-room routing is not a claim of a 243-point transform in this field.

#![forbid(unsafe_code)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]

use continuum_clay_variation::{
    accumulated_drift81, conserved_invariant81, dissipation_weight81, drift_constant81,
    predicted_step81, predicted_total81, prism81, shell_total81, unprism81, ShellModel81, DRIFT81,
    G, K, K_INV, P, W, W_INV,
};

fn residue(value: i128) -> u64 {
    value.rem_euclid(P as i128) as u64
}

fn product(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % P as u128) as u64
}

fn power(base: u64, exponent: u64) -> u64 {
    let mut answer = 1;
    let mut factor = base % P;
    let mut left = exponent;
    while left != 0 {
        if left % 2 != 0 {
            answer = product(answer, factor);
        }
        factor = product(factor, factor);
        left /= 2;
    }
    answer
}

fn total(state: &[u64; K]) -> u64 {
    (state.iter().map(|&x| (x % P) as u128).sum::<u128>() % P as u128) as u64
}

fn weights() -> [u64; K] {
    let mut answer = [0; K];
    let mut weight = 1;
    for slot in &mut answer {
        *slot = weight;
        weight = product(weight, 9);
    }
    answer
}

fn nonlinear(state: &[u64; K]) -> [u64; K] {
    let mut answer = [0; K];
    for (n, slot) in answer.iter_mut().enumerate().take(K - 1).skip(1) {
        let previous = (state[n - 1] % P) as i128;
        let current = (state[n] % P) as i128;
        let next = (state[n + 1] % P) as i128;
        *slot = residue(current * (next - previous));
    }
    let interior = (answer[1..K - 1].iter().map(|&x| x as u128).sum::<u128>() % P as u128) as u64;
    // The implementation splits the canonical residue by ordinary floor, not
    // by multiplication by the inverse of 2 in the finite field.
    answer[0] = residue(-((interior / 2) as i128));
    answer[K - 1] = residue(-((interior - interior / 2) as i128));
    answer
}

fn next_state(state: &[u64; K], nu: u64) -> [u64; K] {
    let transfer = nonlinear(state);
    let factors = weights();
    std::array::from_fn(|n| {
        residue(
            (state[n] % P) as i128 + transfer[n] as i128 - (nu % P) as i128 * factors[n] as i128,
        )
    })
}

fn reference_transform(state: &[u64; K], inverse: bool) -> [u64; K] {
    // Polynomial evaluation in reverse order (Horner), independently of the
    // production forward-order power-accumulation transform.
    let root = power(G, (P - 1) / K as u64);
    let root = if inverse { power(root, P - 2) } else { root };
    let inverse_width = power(K as u64, P - 2);
    std::array::from_fn(|j| {
        let point = power(root, j as u64);
        let mut value = 0u64;
        for &coordinate in state.iter().rev() {
            value = ((value as u128 * point as u128 + (coordinate % P) as u128) % P as u128) as u64;
        }
        if inverse {
            product(value, inverse_width)
        } else {
            value
        }
    })
}

fn sample_state(case: u64) -> [u64; K] {
    // Deterministic polynomial cases independent of the production PRNG.
    std::array::from_fn(|n| {
        let n = n as u128;
        let c = case as u128;
        ((n * n * n * (c + 1) + 104_729 * n + 65_537 * c * c + 1) % P as u128) as u64
    })
}

#[test]
fn audit_prime_field_and_generator_by_trial_division() {
    let mut divisor = 2;
    while divisor <= P / divisor {
        assert_ne!(P % divisor, 0, "divisor={divisor}");
        divisor += 1;
    }
    assert_eq!(P - 1, 2 * 81 * 37 * 167);
    assert_eq!(power(G, P - 1), 1);
    for prime in [2, 3, 37, 167] {
        assert_ne!(power(G, (P - 1) / prime), 1);
    }
}

#[test]
fn audit_transform_constants_and_exact_order() {
    assert_eq!(power(G, (P - 1) / K as u64), W);
    assert_eq!(power(W, 81), 1);
    assert_ne!(power(W, 27), 1);
    assert_eq!(power(W, P - 2), W_INV);
    assert_eq!(power(81, P - 2), K_INV);
    assert_eq!(product(W, W_INV), 1);
    assert_eq!(product(81, K_INV), 1);
}

#[test]
fn audit_existing_243_room_routing_is_not_a_243_point_ntt() {
    assert_eq!((P - 1) % 81, 0);
    assert_eq!((P - 1) % 243, 81);
    assert_eq!(power(W, 243), 1);
    assert_eq!(power(W, 81), 1, "W has order 81, not order 243");
    // This is an arithmetic boundary, not a claim about the availability of
    // existing 243-room hotel groups or three coupled 81-mode ensembles.
}

#[test]
fn audit_all_81_basis_vectors_against_reference_and_inverse() {
    for lane in 0..K {
        let mut basis = [0; K];
        basis[lane] = 1;
        let expected = reference_transform(&basis, false);
        assert_eq!(prism81(&basis), expected, "lane={lane}");
        assert_eq!(unprism81(&expected), basis, "lane={lane}");
        assert_eq!(reference_transform(&expected, true), basis);
    }
}

#[test]
fn audit_full_spectrum_not_only_dc() {
    for case in 0..16 {
        let state = sample_state(case);
        let expected = reference_transform(&state, false);
        assert_eq!(prism81(&state), expected, "case={case}");
        assert_eq!(expected[0], total(&state));
        assert_eq!(unprism81(&expected), state);
    }
}

#[test]
fn audit_inverse_on_independent_spectra() {
    for case in 16..32 {
        let spectrum = sample_state(case);
        let expected = reference_transform(&spectrum, true);
        assert_eq!(unprism81(&spectrum), expected, "case={case}");
        assert_eq!(prism81(&expected), spectrum);
    }
}

#[test]
fn audit_signed_nonlinear_oracle_and_telescoping_edge_flux() {
    let mut states = vec![[0; K], [1; K], [P - 1; K], [u64::MAX; K]];
    states.extend((0..64).map(sample_state));
    for state in states {
        let expected = nonlinear(&state);
        let model = ShellModel81 { u: state, nu: 0 };
        assert_eq!(model.non_linear_transfer(), expected);
        assert_eq!(total(&expected), 0);
        let boundary_flux = residue(
            product(state[K - 2], state[K - 1]) as i128 - product(state[0], state[1]) as i128,
        );
        assert_eq!(
            residue(expected[1..K - 1].iter().map(|&x| x as i128).sum()),
            boundary_flux
        );
    }
}

#[test]
fn audit_endpoint_floor_split_on_odd_and_extreme_residues() {
    for interior in [0, 1, 2, 3, P - 3, P - 2, P - 1] {
        let mut state = [0; K];
        state[K - 2] = 1;
        state[K - 1] = interior;
        let expected = nonlinear(&state);
        let actual = ShellModel81 { u: state, nu: 0 }.non_linear_transfer();
        assert_eq!(actual, expected, "S={interior}");
        assert_eq!(actual[0], residue(-((interior / 2) as i128)));
        assert_eq!(actual[K - 1], residue(-((interior - interior / 2) as i128)));
    }
    let mut odd = [0; K];
    odd[K - 2] = 1;
    odd[K - 1] = 1;
    assert_eq!(nonlinear(&odd)[0], 0);
    assert_ne!(nonlinear(&odd)[0], residue(-((P + 1) as i128 / 2)));
}

#[test]
fn audit_iterated_weights_and_drift_without_production_pow() {
    let expected = weights();
    for (n, &weight) in expected.iter().enumerate() {
        assert_eq!(dissipation_weight81(n), weight, "rung={n}");
    }
    let drift = total(&expected);
    assert_eq!(drift, 900_647);
    assert_eq!(DRIFT81, drift);
    assert_eq!(drift_constant81(), drift);
    assert_eq!(
        residue(8 * drift as i128),
        residue(power(9, 81) as i128 - 1)
    );
}

#[test]
fn audit_step_coordinates_for_viscosity_edges() {
    for nu in [0, 1, P - 1, P, P + 1, u64::MAX] {
        for case in 0..24 {
            let state = sample_state(case);
            let expected = next_state(&state, nu);
            let mut model = ShellModel81 { u: state, nu };
            model.step();
            assert_eq!(model.u, expected, "nu={nu}|case={case}");
            assert_eq!(predicted_step81(&state, nu), expected);
            assert!(model.u.iter().all(|&x| x < P));
        }
    }
}

#[test]
fn audit_multistep_oracle_and_all_time_phase_formula() {
    let drift = total(&weights());
    for nu in [0, 1, P - 1, P, P + 1, u64::MAX] {
        for case in 0..8 {
            let initial = sample_state(case);
            let q0 = total(&initial);
            let mut expected = initial;
            let mut model = ShellModel81 { u: initial, nu };
            for time in 1..=32 {
                expected = next_state(&expected, nu);
                model.step();
                assert_eq!(model.u, expected, "nu={nu}|case={case}|t={time}");
                let accumulated = product(product(time, nu), drift);
                let expected_total = residue(q0 as i128 - accumulated as i128);
                assert_eq!(total(&model.u), expected_total);
                assert_eq!(shell_total81(&model.u), expected_total);
                assert_eq!(predicted_total81(q0, nu, time), expected_total);
                assert_eq!(accumulated_drift81(nu, time), accumulated);
                assert_eq!(conserved_invariant81(&model.u, nu, time), q0);
            }
        }
    }
}

#[test]
fn audit_scalar_phase_period_is_one_when_nu_is_zero_mod_p() {
    let q0 = 123_456;
    for nu in [0, P, 2 * P, 3 * P] {
        for time in [0, 1, 2, P - 1, P, P + 1, u64::MAX] {
            assert_eq!(accumulated_drift81(nu, time), 0);
            assert_eq!(predicted_total81(q0, nu, time), q0);
        }
    }
}

#[test]
fn audit_nonzero_scalar_drift_has_exact_period_p() {
    // Exhaust every time residue for nu=1; no model trajectory is substituted
    // for this scalar closed-form phase sequence.
    let q0 = 765_432;
    let mut seen = vec![false; P as usize];
    for time in 0..P {
        let value = predicted_total81(q0, 1, time);
        let expected = residue(q0 as i128 - time as i128 * DRIFT81 as i128);
        assert_eq!(value, expected);
        assert!(!seen[value as usize], "repeat at t={time}");
        seen[value as usize] = true;
    }
    assert!(seen.into_iter().all(|visited| visited));
    assert_eq!(predicted_total81(q0, 1, P), q0);
    for nu in [1, P - 1, P + 1, u64::MAX] {
        assert_ne!(product(nu, DRIFT81), 0);
        assert_eq!(predicted_total81(q0, nu, P), q0);
        assert_ne!(predicted_total81(q0, nu, 1), q0);
    }
}

#[test]
fn audit_scalar_period_does_not_assert_full_state_period() {
    let mut initial = [0; K];
    initial[0] = 1;
    initial[1] = 1;
    let mut model = ShellModel81 { u: initial, nu: 0 };
    model.step();
    assert_eq!(total(&model.u), total(&initial));
    assert_ne!(model.u, initial, "Q has period 1; this state does not");
    assert_eq!(model.u, next_state(&initial, 0));
}

#[test]
fn audit_positive_viscosity_forces_zero_unless_zero_mod_p() {
    let zero = [0; K];
    let mut model = ShellModel81 { u: zero, nu: 1 };
    model.step();
    assert_eq!(model.u, next_state(&zero, 1));
    assert_eq!(model.u[0], P - 1);
    assert!(model.u.iter().all(|&x| x != 0));
    assert_eq!(total(&model.u), residue(-(DRIFT81 as i128)));
    let mut positive_multiple = ShellModel81 { u: zero, nu: P };
    positive_multiple.step();
    assert_eq!(positive_multiple.u, zero);
}

#[test]
fn audit_linear_balance_is_not_quadratic_energy_cancellation() {
    let mut state = [0; K];
    state[0] = 1;
    state[1] = 1;
    let transfer = nonlinear(&state);
    let dot = residue(
        state
            .iter()
            .zip(transfer)
            .map(|(&x, y)| x as i128 * y as i128)
            .sum(),
    );
    assert_eq!(total(&transfer), 0);
    assert_eq!(dot, (P - 1) / 2);
    assert_ne!(dot, 0);
    assert_eq!(
        ShellModel81 { u: state, nu: 0 }.non_linear_transfer(),
        transfer
    );
}

#[test]
fn audit_transform_inverse_does_not_imply_dynamical_injectivity() {
    let zero = [0; K];
    let mut nonzero = [0; K];
    nonzero[1] = 1;
    nonzero[2] = P - 1;
    assert_ne!(nonzero, zero);
    for nu in [0, 1, P, u64::MAX] {
        assert_eq!(next_state(&nonzero, nu), next_state(&zero, nu));
        let mut a = ShellModel81 { u: nonzero, nu };
        let mut b = ShellModel81 { u: zero, nu };
        a.step();
        b.step();
        assert_eq!(a.u, b.u, "same image of different states|nu={nu}");
    }
    assert_eq!(unprism81(&prism81(&nonzero)), nonzero);
}
