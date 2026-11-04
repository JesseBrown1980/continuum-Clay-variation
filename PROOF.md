# The 81-Mode Continuum Clay Variation: All-Time Invariant Theorem & Scaling Analysis

> **Corrected-scope notice (2026-11-04).** This document records the original derivation. The independent [Clasy X3 proof](docs/CLASY-X3-PROOF.md) is the current reference for the implemented finite-field map and its limitations. The term called "viscosity" in the code is an additive coordinate-dependent drift, not the Navier–Stokes velocity-dependent viscous operator. The finite-field conserved total and bounded modular residues do not establish continuum PDE regularity, energy dissipation, or a physical singularity bound.

This repository presents the **81-Mode Continuum Clay Variation** of the 3-adic shell model, extending the base 27-mode system into a 4D hypercube lattice ($n \times 3 \times 3 \times 3$ architecture with $n = 3$, giving $K = 3^4 = 81$ modes).

It incorporates Asolaria's balanced ternary tribit architecture, the canonical Asolaria tribit kernel (ASI OS commit `c0dfee6`), an exact 81-point spherical Number-Theoretic Transform (NTT) prism, and an 81-trit Omega/Omnibit room rotor representation across 81 geometric rungs ($k_n = 3^n$, $n \in [0, 80]$).

---

## 1. Mathematical Architecture & Setting

### The Geometric 3-Ladder
Wavenumbers are organized on the geometric ladder:
$$k_n = 3^n, \qquad n \in [0, 80]$$

In 27-mode space, the highest wavenumber is $k_{26} = 3^{26} \approx 2.54 \times 10^{12}$.
In 81-mode space, the wavenumbers reach:
$$k_{80} = 3^{80} \approx 1.47 \times 10^{38}$$
This spans over 38 orders of magnitude across spatial scales, reaching deep into the ultra-high frequency sub-continuum.

### The 81-Mode Prime Field $\mathbb{F}_P$
To define a primitive 81st root of unity for the spherical NTT prism, the field modulus $P$ must satisfy $81 \mid (P - 1)$.
We fix the prime modulus:
$$P = 1\,000\,999$$
Where $P - 1 = 1\,000\,998 = 2 \times 3^4 \times 37 \times 167 = 81 \times 12\,358$.

The generator of $\mathbb{F}_P^\times$ is $G = 6$. The primitive 81st root of unity is:
$$W = G^{(P-1)/81} \bmod P = 6^{12\,358} \bmod 1\,000\,999 = 30\,722$$
With multiplicative inverse $W^{-1} = 341\,106 \pmod P$.

### Exact 81-Point Spherical NTT Prism
The forward transform (`prism81`) and inverse transform (`unprism81`) over $(\mathbb{Z}/P\mathbb{Z})^{81}$ are given by:
$$X_j = \sum_{n=0}^{80} x_n W^{jn} \pmod P, \qquad x_n = 81^{-1} \sum_{j=0}^{80} X_j W^{-jn} \pmod P$$
Where $81^{-1} \equiv 988\,641 \pmod P$.

- **Law 20 Cyclotomic Closure:**
  $$\sum_{j=0}^{80} W^j \equiv 0 \pmod P$$
- **Law 0 DC Center Invariant:**
  $$X_0 = \sum_{n=0}^{80} x_n \pmod P$$
- **Bit-Exact Invertibility:**
  $$\text{unprism81}(\text{prism81}(x)) \equiv x, \quad \forall x \in \mathbb{F}_P^{81}$$

---

## 2. Triad Convective Dynamics & Exact Zero-Sum Conservation

### The 81-Mode Transfer
For interior rungs $1 \le n \le 79$:
$$NL_n = u_n (u_{n+1} - u_{n-1}) \pmod P$$
Let $S = \sum_{n=1}^{79} NL_n \pmod P$. The boundary modes are assigned:
$$NL_0 = -\lfloor S/2 \rfloor \pmod P, \qquad NL_{80} = -S + \lfloor S/2 \rfloor \pmod P$$

### Lemma 1: Exact Total Transfer Cancellation
For every state $u \in \mathbb{F}_P^{81}$:
$$\sum_{n=0}^{80} NL_n = -\lfloor S/2 \rfloor + S - S + \lfloor S/2 \rfloor = 0 \pmod P$$
Under the forward spherical NTT prism:
$$X[0] = \text{prism81}(NL)[0] = \sum_{n=0}^{80} NL_n \equiv 0 \pmod P$$
The convective flux creates zero net DC accumulation across the entire 81-mode sphere.

---

## 3. Viscous Dissipation & The All-Time Invariant Theorem

### Lemma 2: Closed-Form Step
At each rung $n \in [0, 80]$, the discrete time step computes:
$$u_n^{(t+1)} \equiv u_n^{(t)} + NL_n(u^{(t)}) - \nu \cdot (3^n)^2 \pmod P$$
Because $(3^n)^2 = 9^n$, the subtracted term is $\nu \cdot 9^n \pmod P$.

### The 81-Mode Viscous Drift Constant
Summing the viscous dissipation factors across all 81 modes:
$$D_{81} = \sum_{n=0}^{80} 9^n \bmod P = 900\,647$$
Because $9^{81} \equiv 198\,184 \pmod P$, the geometric series identity holds:
$$8 \cdot D_{81} \equiv 9^{81} - 1 \equiv 198\,183 \pmod P$$
Since $D_{81} \ne 0 \pmod P$, the total has a nonzero drift **only when** $\nu\not\equiv0\pmod P$; in particular, a positive multiple of $P$ gives zero drift.

### The All-Time Invariant Theorem
For any initial velocity field $u^{(0)} \in \mathbb{F}_P^{81}$ evolving under $u^{(t+1)} = \text{step}(u^{(t)})$, the shell total $Q(u) = \sum_{n=0}^{80} u_n \pmod P$ satisfies, for every $t \ge 0$:
$$Q(u^{(t)}) \equiv Q(u^{(0)}) - t \cdot \nu \cdot D_{81} \pmod P$$
And the quantity:
$$I(t, u^{(t)}) := Q(u^{(t)}) + t \cdot \nu \cdot D_{81} \equiv Q(u^{(0)}) \pmod P$$
is conserved for all time $t \ge 0$. $\blacksquare$

### Corollaries:
1. **Inviscid Conservation ($\nu = 0$):** If $\nu = 0$, $Q(u^{(t)}) \equiv Q(u^{(0)})$ is strictly conserved for all $t$.
2. **Scalar period (when $\nu\not\equiv0\pmod P$):** Because $P$ is prime and $D_{81}\not\equiv0\pmod P$, $t\mapsto Q(u^{(t)})$ visits each residue exactly once per $P$ steps. When $\nu\equiv0\pmod P$ (including positive multiples of $P$), the scalar total is constant. This is a statement about $Q$, not the period of the full state.

---

## 4. The $n \times 3 \times 3 \times 3$ Omega / Omnibit Room Architecture

An 81-trit room corresponds to $n = 3$ clusters of $3 \times 3 \times 3 = 27$ trits:
- **Alpha stub (trits 0..26):** Macro-scale low frequencies ($n \in [0, 26]$).
- **Omega stub (trits 27..53):** Inertial cascade & intermediate dissipation ($n \in [27, 53]$).
- **Omnibit stub (trits 54..80):** Deep viscous suppression ($n \in [54, 80]$).

Each 81-trit room is encoded in 24 bytes (`[u64; 3]`).
Across the tested Host8 ensembles and procedural room seeds:
- Convective momentum flux: $\sum NL_n \equiv 0 \pmod P$ (100.000% pass).
- DC center invariant: $X[0] \equiv 0 \pmod P$ (100.000% pass).
- Invariant conservation: $I(t, u_t) \equiv Q(u_0) \pmod P$ (100.000% pass).

---

## 5. The Continuum Clay Variation & Beale-Kato-Majda Analysis

### Scaling labels versus physical damping

The formal shell labels are $k_n=3^n$. The implemented recurrence subtracts $\nu 9^n \pmod P$ **without multiplying by $u_n$**. Consequently the ratio of the *unreduced labels* $9^n/3^n=3^n$ is not a rigorous balance between viscous damping and nonlinear energy transport. Finite-field residues do not carry a monotone order in which one can infer dissipative domination. The original "viscous domination" claim is superseded by [Clasy X3, Section 6](docs/CLASY-X3-PROOF.md#6-unreduced-norms-and-continuum-regularity).

### The Discrete-to-Continuum Bridge
The [Clay Mathematics Millennium Prize problem](https://www.claymath.org/wp-content/uploads/2022/06/navierstokes.pdf) requires proving smooth solutions in $\mathbb{R}^3$ for all time under the Beale-Kato-Majda (BKM) criterion:
$$\int_0^T \|\omega(\cdot, t)\|_{L^\infty} dt < \infty \quad \forall T < \infty$$

The finite-field implementation reports the **modular residue**
$$\Omega_{\text{field}}(t) = \left(\frac{1}{2}\sum_{n=0}^{80} 9^n u_n(t)^2\right) \bmod P < P$$
This is a residue identity, not a physical vorticity bound. A complete PDE proof would require:
1. Reconstructing a continuous divergence-free velocity field $u(x, t) \in C^\infty(\mathbb{R}^3)$ from the 81 shell amplitudes.
2. Proving that the unreduced Sobolev $H^s$ norms remain bounded independently of $P$ and $K$.
3. Establishing that the discrete-to-continuum limit $K \to \infty$ converges strongly to a classical solution of the Navier-Stokes PDE.

The 81-mode variation provides an exact, reproducible discrete foundation across 81 rungs, verifying that the algebraic center invariant $X[0] \equiv 0$ and the all-time drift law $I = Q + t\nu D_{81}$ remain unconditional across all 81 modes.
