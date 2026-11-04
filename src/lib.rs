//! Continuum Clay Variation: 81-Mode 3-Adic Shell Model & Omega/Omnibit Invariant Architecture.
//!
//! # The 81-Mode Continuum Extension
//! Expanding the 27-mode shell system ($K = 27 = 3^3$) to 81 modes ($K = 81 = 3^4$)
//! covers 81 rungs of the geometric 3-ladder ($k_n = 3^n$, $n \in [0, 80]$).
//! Wavenumbers span from $k_0 = 1$ to $k_{80} = 3^{80} \approx 1.47 \times 10^{38}$,
//! bridging 38 orders of magnitude across spatial scales into the continuum limit.
//!
//! # Mathematical Foundations
//! - Prime field: $\mathbb{F}_P$ with $P = 1\,000\,999$ ($81 \mid (P - 1)$, where $P - 1 = 81 \times 12\,358$).
//! - Primitive 81st root of unity: $W = 30\,722$ with $W^{81} \equiv 1 \pmod P$ and $W^{27} \not\equiv 1 \pmod P$.
//! - Exact 81-point spherical NTT prism: `prism81` and `unprism81` with bit-exact integer round-trip.
//! - Non-linear triad transfer: $\sum_{n=0}^{80} NL_n \equiv 0 \pmod P$ identically for all states in $\mathbb{F}_P^{81}$.
//! - Law 0 DC Center Invariant: $X[0] = \text{prism81}(NL)[0] \equiv \sum_{n=0}^{80} NL_n \equiv 0 \pmod P$.
//! - Viscous drift constant: $D_{81} = \sum_{n=0}^{80} 9^n \equiv 900\,647 \pmod P$.
//! - All-time conserved law: $I(t, u_t) = Q(u_t) + t \cdot \nu \cdot D_{81} \equiv Q(u_0) \pmod P$.

#![forbid(unsafe_code)]
#![allow(clippy::needless_range_loop)]
#![deny(
    clippy::float_arithmetic,
    clippy::float_cmp,
    clippy::cast_precision_loss
)]

pub use navier_stokes_regularity_spectral as upstream27;

/// Canonical Asolaria tribit module (ASI-OS commit c0dfee6), included unchanged.
#[path = "../vendor/canonical/tribit/mod.rs"]
pub mod canonical_tribit;

pub use canonical_tribit::Zero;

/// Read-only Host8 evaluation adapter; the shell equations above remain unchanged.
pub mod host8_audit;

// ---------------------------------------------------------------- Constants

/// The 81-mode prime field modulus: P = 1_000_999 (prime, 81 | P - 1).
pub const P: u64 = 1_000_999;
/// Generator of F_P*.
pub const G: u64 = 6;
/// Number of shell modes: K = 81 = 3^4.
pub const K: usize = 81;
/// Primitive 81st root of unity: W = G^((P-1)/81) mod P.
pub const W: u64 = 30_722;
/// Multiplicative inverse of W mod P.
pub const W_INV: u64 = 341_106;
/// Multiplicative inverse of K (81) mod P.
pub const K_INV: u64 = 988_641;
/// Viscous dissipation drift constant: D = sum_{n=0}^{80} 9^n mod P = 900_647.
pub const DRIFT81: u64 = 900_647;

// 27-mode backward compatibility constants
pub const P27: u64 = 1_000_081;
pub const K27: usize = 27;
pub const DRIFT27: u64 = 883_973;

// ---------------------------------------------------------------- Modular Arithmetic

#[inline]
pub const fn mul_mod(a: u64, b: u64) -> u64 {
    ((a as u128 * b as u128) % P as u128) as u64
}

#[inline]
pub const fn add_mod(a: u64, b: u64) -> u64 {
    let s = a + b;
    if s >= P {
        s - P
    } else {
        s
    }
}

#[inline]
pub const fn sub_mod(a: u64, b: u64) -> u64 {
    let a = a % P;
    let b = b % P;
    if a >= b {
        a - b
    } else {
        a + P - b
    }
}

pub const fn pow_mod(mut base: u64, mut exp: u64) -> u64 {
    let mut acc: u64 = 1;
    base %= P;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = mul_mod(acc, base);
        }
        base = mul_mod(base, base);
        exp >>= 1;
    }
    acc
}

#[inline]
pub const fn inv_mod(a: u64) -> u64 {
    pow_mod(a, P - 2)
}

/// Computes 9^n mod P (dissipation weight for rung n).
pub const fn dissipation_weight81(n: usize) -> u64 {
    let mut acc = 1u64;
    let mut i = 0;
    while i < n {
        acc = mul_mod(acc, 9);
        i += 1;
    }
    acc
}

/// Recomputes the drift constant D81 from its mathematical definition.
pub const fn drift_constant81() -> u64 {
    let mut acc = 0u64;
    let mut n = 0;
    while n < K {
        acc = add_mod(acc, dissipation_weight81(n));
        n += 1;
    }
    acc
}

// ---------------------------------------------------------------- 81-Point Spherical NTT Prism

/// Forward 81-point spherical Number-Theoretic Transform over F_P.
pub fn prism81(x: &[u64; K]) -> [u64; K] {
    let mut out = [0u64; K];
    for (j, slot) in out.iter_mut().enumerate() {
        let wj = pow_mod(W, j as u64);
        let mut acc = 0u64;
        let mut t = 1u64;
        for &val in x {
            acc = add_mod(acc, mul_mod(val % P, t));
            t = mul_mod(t, wj);
        }
        *slot = acc;
    }
    out
}

/// Inverse 81-point spherical Number-Theoretic Transform over F_P.
pub fn unprism81(x: &[u64; K]) -> [u64; K] {
    let mut out = [0u64; K];
    for (j, slot) in out.iter_mut().enumerate() {
        let wj = pow_mod(W_INV, j as u64);
        let mut acc = 0u64;
        let mut t = 1u64;
        for &val in x {
            acc = add_mod(acc, mul_mod(val % P, t));
            t = mul_mod(t, wj);
        }
        *slot = mul_mod(acc, K_INV);
    }
    out
}

// ---------------------------------------------------------------- Omega / Omnibit 81-Trit Room Representation

/// A 24-byte Omega-Omnibit Room holding 81 balanced trits across 3 rungs:
/// - Alpha: trits 0..26 (low rime frequencies)
/// - Omega: trits 27..53 (mid inertio-viscous transition)
/// - Omni:  trits 54..80 (ultra-high dissipative sub-continuum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmegaOmnibitRoom {
    pub alpha: u64,
    pub omega: u64,
    pub omni: u64,
}

impl OmegaOmnibitRoom {
    pub fn new(alpha: u64, omega: u64, omni: u64) -> Self {
        Self { alpha, omega, omni }
    }

    /// Packs into 24 bytes (canonical little-endian).
    pub fn to_bytes(self) -> [u8; 24] {
        let mut bytes = [0u8; 24];
        bytes[0..8].copy_from_slice(&self.alpha.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.omega.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.omni.to_le_bytes());
        bytes
    }

    /// Unpacks from 24 bytes.
    pub fn from_bytes(bytes: &[u8; 24]) -> Self {
        let mut a = [0u8; 8];
        let mut o = [0u8; 8];
        let mut m = [0u8; 8];
        a.copy_from_slice(&bytes[0..8]);
        o.copy_from_slice(&bytes[8..16]);
        m.copy_from_slice(&bytes[16..24]);
        Self {
            alpha: u64::from_le_bytes(a),
            omega: u64::from_le_bytes(o),
            omni: u64::from_le_bytes(m),
        }
    }

    /// Unpacks the 81 trits into an 81-mode velocity vector in F_P.
    pub fn unpack_state(self, base_energy: u64) -> [u64; K] {
        let mut u = [0u64; K];
        let stubs = [self.alpha, self.omega, self.omni];

        for (stub_idx, &stub) in stubs.iter().enumerate() {
            let mut v = stub;
            for rung in 0..27 {
                let lane = stub_idx * 27 + rung;
                let z = Zero::from_digit((v % 3) as u8);
                v /= 3;
                let amp = match z {
                    Zero::Neg => sub_mod(0, base_energy % P),
                    Zero::Nil => 0,
                    Zero::Pos => base_energy % P,
                };
                let salt = (stub.wrapping_add(lane as u64)) % 10_000;
                u[lane] = add_mod(amp, salt % P);
            }
        }
        u
    }
}

// ---------------------------------------------------------------- 81-Mode Discrete Shell Model

/// 81-mode discrete 3-adic shell model of turbulence.
#[derive(Clone, Debug)]
pub struct ShellModel81 {
    pub u: [u64; K],
    pub nu: u64,
}

impl ShellModel81 {
    pub fn new(initial_energy: u64, nu: u64) -> Self {
        let mut u = [0u64; K];
        u[0] = initial_energy % P;
        u[1] = (initial_energy / 2) % P;
        u[2] = (initial_energy / 4) % P;
        Self { u, nu }
    }

    pub fn from_room(room: OmegaOmnibitRoom, nu: u64, base_energy: u64) -> Self {
        let u = room.unpack_state(base_energy);
        Self { u, nu }
    }

    /// Triad convective transfer across 81 modes:
    /// NL_n = u_n (u_{n+1} - u_{n-1}) mod P for 1 <= n <= 79
    /// Endpoints NL_0 and NL_80 balance the interior sum identically.
    pub fn non_linear_transfer(&self) -> [u64; K] {
        let mut nl = [0u64; K];
        for n in 1..K - 1 {
            let u_prev = self.u[n - 1] as u128;
            let u_curr = self.u[n] as u128;
            let u_next = self.u[n + 1] as u128;

            let term1 = (u_next * u_curr) % P as u128;
            let term2 = (u_prev * u_curr) % P as u128;

            let transfer = (term1 + P as u128 - term2) % P as u128;
            nl[n] = transfer as u64;
        }

        let mut sum_interior = 0u128;
        for &item in nl.iter().take(K - 1).skip(1) {
            sum_interior = (sum_interior + item as u128) % P as u128;
        }
        nl[0] = ((P as u128 - sum_interior / 2) % P as u128) as u64;
        nl[K - 1] = ((P as u128 - sum_interior + sum_interior / 2) % P as u128) as u64;

        nl
    }

    /// Discrete forward time step:
    /// u_n^(t+1) = u_n + NL_n - nu * 9^n mod P.
    pub fn step(&mut self) {
        let nl = self.non_linear_transfer();
        let nu = self.nu % P;
        for (n, (u_slot, &nl_val)) in self.u.iter_mut().zip(nl.iter()).enumerate() {
            let diss = mul_mod(nu, dissipation_weight81(n));
            let un_new = sub_mod(add_mod(*u_slot % P, nl_val), diss);
            *u_slot = un_new;
        }
    }

    /// Q(u) = sum_{n=0}^{80} u_n mod P.
    pub fn shell_total(&self) -> u64 {
        shell_total81(&self.u)
    }

    /// Modular kinetic energy: (1/2 * sum u_n^2) mod P.
    pub fn total_energy(&self) -> u64 {
        let mut acc = 0u128;
        for &un_val in &self.u {
            let un = un_val as u128;
            acc = (acc + un * un) % P as u128;
        }
        let half = ((P + 1) / 2) as u128;
        ((acc * half) % P as u128) as u64
    }

    /// Modular enstrophy: (1/2 * sum 9^n u_n^2) mod P.
    pub fn total_enstrophy(&self) -> u64 {
        let mut acc = 0u128;
        for (n, &un_val) in self.u.iter().enumerate() {
            let un = un_val as u128;
            let kn2 = dissipation_weight81(n) as u128;
            acc = (acc + kn2 * ((un * un) % P as u128)) % P as u128;
        }
        let half = ((P + 1) / 2) as u128;
        ((acc * half) % P as u128) as u64
    }

    /// Unreduced integer energy sum: sum_{n=0}^{80} u_n^2 over u128.
    /// Exposes genuine growth vs modular residue reduction.
    pub fn unreduced_energy_u128(&self) -> u128 {
        let mut acc = 0u128;
        for &un_val in &self.u {
            let un = (un_val % P) as u128;
            acc += un * un;
        }
        acc
    }
}

// ---------------------------------------------------------------- Exact Invariant Formulation

/// Shell-coordinate sum Q(u) = sum_{n=0}^{80} u_n mod P.
pub fn shell_total81(u: &[u64; K]) -> u64 {
    u.iter().fold(0u64, |acc, &x| add_mod(acc, x % P))
}

/// Closed-form single step prediction for 81 modes:
pub fn predicted_step81(u: &[u64; K], nu: u64) -> [u64; K] {
    let model = ShellModel81 { u: *u, nu };
    let nl = model.non_linear_transfer();
    let nu = nu % P;
    let mut out = [0u64; K];
    for n in 0..K {
        out[n] = sub_mod(
            add_mod(u[n] % P, nl[n]),
            mul_mod(nu, dissipation_weight81(n)),
        );
    }
    out
}

/// Accumulated viscous drift: (t * nu * DRIFT81) mod P.
pub fn accumulated_drift81(nu: u64, t: u64) -> u64 {
    mul_mod(mul_mod(t % P, nu % P), DRIFT81)
}

/// Predicted shell total after t steps: Q(t) = Q(0) - t * nu * DRIFT81 mod P.
pub fn predicted_total81(q0: u64, nu: u64, t: u64) -> u64 {
    sub_mod(q0, accumulated_drift81(nu, t))
}

/// Conserved all-time quantity: I(t, u) = Q(u) + t * nu * DRIFT81 mod P == Q(0).
pub fn conserved_invariant81(u: &[u64; K], nu: u64, t: u64) -> u64 {
    add_mod(shell_total81(u), accumulated_drift81(nu, t))
}

// ---------------------------------------------------------------- PRNG for reproducible test ensembles

pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    pub fn state81(&mut self) -> [u64; K] {
        let mut u = [0u64; K];
        for slot in &mut u {
            *slot = self.next_u64() % P;
        }
        u
    }

    pub fn room(&mut self) -> OmegaOmnibitRoom {
        OmegaOmnibitRoom {
            alpha: self.next_u64(),
            omega: self.next_u64(),
            omni: self.next_u64(),
        }
    }
}
