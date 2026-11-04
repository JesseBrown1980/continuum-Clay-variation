# Continuum Clay Variation
### 81-mode finite-field shell system · exact algebra · independently checked implementation

[![Rust 1.81](https://img.shields.io/badge/Rust-1.81-0b1320?logo=rust)](rust-toolchain.toml)
[![Clasy X3](https://img.shields.io/badge/Clasy%20X3-73%20tests%20recorded-138a9d)](https://github.com/JesseBrown1980/Clasy-repo-x-3-)
[![Finite field](https://img.shields.io/badge/Field-%E2%84%A4%2F1%2C000%2C999-303d92)](docs/CLASY-X3-PROOF.md)
[![Math scope](https://img.shields.io/badge/continuum%20PDE-OPEN-965d2d)](#scope-and-corrections)

<table><tr>
<td width="50%"><img src="assets/finite-shell-vortex.svg" alt="Original abstract blue and cyan luminous vortex, an illustrative fluid-dynamics motif" width="100%"></td>
<td width="50%"><img src="assets/finite-shell-spectral-lattice.svg" alt="Original cyan and green finite-field spectral wireframe, an illustrative mathematical motif" width="100%"></td>
</tr></table>

*Original repository-owned vector illustrations. These depict the research themes, not measured velocity fields or actual experiment plots.*

**This repository defines an exact, finite 81-mode nonlinear dynamical system over a prime field.** The companion [Clasy X3 independent evaluation](https://github.com/JesseBrown1980/Clasy-repo-x-3-) corrects the model's interpretation, supplies independent arithmetic oracles and negative controls, and evaluates stored Host8 rooms. This update ports those checks into the originating Clay repository **without changing the implemented shell update**.

> **Scope:** The all-time modular invariant is a theorem **of this specific finite-field map**. It is not a proof of Navier–Stokes regularity, viscosity-driven physical damping, or convergence to a continuum PDE. The original research name is retained for provenance.

## The exact model

| Parameter | Definition |
|---|---|
| Field | $\mathbb F_P$, $P=1{,}000{,}999$ (prime) |
| Coordinates | $u=(u_0,\dots,u_{80})\in\mathbb F_P^{81}$ |
| Geometric labels | $k_n=3^n$, $0\le n\le80$ |
| Primitive 81st root | $W=30{,}722\pmod P$ |
| Transform | Reversible 81-point number-theoretic transform |
| Drift constant | $D_{81}=\sum_{n=0}^{80}9^n=900{,}647\pmod P$ |

For the interior coordinates ($1\le n\le79$) define

$$
NL_n(u)=u_n(u_{n+1}-u_{n-1})\pmod P.
$$

Let $S$ be the **canonical integer residue** of the sum of the interior $NL_n$. Set

$$
NL_0=-\lfloor S/2\rfloor,\qquad
NL_{80}=-S+\lfloor S/2\rfloor \pmod P.
$$

Those endpoints make $\sum_{n=0}^{80}NL_n=0\pmod P$ **for every state**, by construction. One step is

$$
u_n(t+1)=u_n(t)+NL_n(u(t))-\nu\,9^n\pmod P.
$$

Defining $Q(u)=\sum_n u_n\pmod P$, summing the update proves the **all-time theorem**:

$$
\boxed{Q(u(t))+t\nu D_{81}=Q(u(0))\pmod P,\qquad t\in\mathbb Z_{\ge0}.}
$$

The zero-frequency transform coordinate of $NL$ is also identically zero. The NTT is invertible, but **the nonlinear time-step map need not be injective**.

## What Clasy contributed

The independent Clasy evaluation, pinned to [upstream commit 12b1bd4](https://github.com/JesseBrown1980/continuum-Clay-variation/commit/12b1bd4ed407a9d7e622182337f54d84daaebc12), supplies:

- **Independent integer arithmetic oracles** for the field, NTT, nonlinear endpoints, time evolution, period, and previously missed edge cases: [tests/audit81.rs](tests/audit81.rs).
- **Fail-closed I/O tests** covering absent stores, truncated sectors, invalid CLI parameters, endianness and short reads: [tests/fail_closed.rs](tests/fail_closed.rs).
- **Read-only 81/243-width Host8 auditor**, with complete-coverage accounting on its selected windows: [src/host8_audit.rs](src/host8_audit.rs) and [src/bin/audit_rooms.rs](src/bin/audit_rooms.rs).
- **The existing 3/27/243 transition-lattice contract**, brought into this repository with its two vendored lattice source files: [src/bin/sector_contract.rs](src/bin/sector_contract.rs).
- **Corrected mathematical scope**, including a dynamical non-injectivity counterexample and unreduced energy cautions: [Clasy X3 proof](docs/CLASY-X3-PROOF.md).

### Reported independent verification (Clasy, October 23, 2026)

| Recorded check | Clasy result | Qualification |
|---|---:|---|
| Rust tests | **73 passed, 0 failed** | Recorded in the [Clasy test log](https://github.com/JesseBrown1980/Clasy-repo-x-3-/blob/main/logs/x3/tests.log); this branch has its own CI |
| NTT round trips | **10,000** | Exact finite-field equality |
| Full-period model trajectories | **4 × 1,000,999 steps** | Total-period law, not full-state periodicity |
| Scalar-phase census | **1,000,999 distinct residues** | When $\nu\not\equiv0\pmod P$ |
| Host8 room reads | **1,093,500** | Sampled windows over 375 physical sectors, not all 37.5 billion rooms |
| Selected state transitions | **1,093,500** | Read-only input; three steps per ensemble |
| 3/27/243 lattice keys | **4,095** | In-process contract callbacks, not external worker launches |

The figures above are **historical Clasy receipts**, not numbers newly claimed to have run on this PR. See the original [Clasy results](https://github.com/JesseBrown1980/Clasy-repo-x-3-/blob/main/RESULTS.md) and [receipts](https://github.com/JesseBrown1980/Clasy-repo-x-3-/tree/main/receipts/x3).

## Scope and corrections

| Statement | Status |
|---|---|
| Explicit 81-mode map, NTT constants, endpoint closure | **DERIVED** for the defined system |
| Compensated sum $Q+t\nu D_{81}$ | **DERIVED for all integer times** |
| Scalar total visits every field residue | **DERIVED only when** $\nu\not\equiv0\pmod P$ |
| Full-state time-step injectivity | **FALSE**; Clasy gives a two-state counterexample |
| Quadratic kinetic-energy conservation | **NOT IMPLIED** by linear total cancellation |
| Standard Navier–Stokes viscous damping | **NOT IMPLEMENTED**: the $-\nu9^n$ term is additive, independent of $u_n$ |
| Physical divergence-free velocity reconstruction | **OPEN** |
| Consistency and convergence to 3D Navier–Stokes | **OPEN** |
| Continuum regularity / Clay Millennium claim | **OPEN** |

The field modulus bounds residues; it does **not** bound physical Sobolev or vorticity norms uniformly as model resolution grows. A 243-room read width is **not** a primitive 243-point NTT in this field.

## Reproduce

The ported core suite is designed to run **without** a Host8 disk. Requires Rust 1.81, Clippy and rustfmt:

~~~bash
rustup toolchain install 1.81.0 --profile minimal --component clippy --component rustfmt
bash scripts/x3-ci.sh
~~~

The core gate formats, lints, builds, and runs all tests. To run the full-period verifier as well:

~~~bash
RUN_FULL_VERIFY=1 bash scripts/x3-ci.sh
~~~

For the **optional real Host8 storage audit**, on a machine with the canonical 300 GB room store:

~~~bash
bash scripts/x3-gate.sh /mnt/d/ASOLARIA-HOST8-ROOMS-300G
~~~

The Host8 workflow does not generate substitute rooms, and fails if the required store is missing or malformed. The historical standalone rotor sweep remains available via [scripts/gate.sh](scripts/gate.sh).

## Research route

A physical correspondence would require a divergence-free field reconstruction and boundary/domain specification, a correctly modeled velocity-dependent viscosity term and forcing, consistency estimates, convergence as resolution increases, and **unreduced** norm/continuation bounds. These remain research tasks rather than completed claims.

**Documents:** [Corrected proof](docs/CLASY-X3-PROOF.md) · [Clasy measured results](https://github.com/JesseBrown1980/Clasy-repo-x-3-/blob/main/RESULTS.md) · [Original proof and historical claims](PROOF.md) · [Current tests](tests/audit81.rs) · [Original Clasy repository](https://github.com/JesseBrown1980/Clasy-repo-x-3-).

**Provenance:** Research and architecture attributed to Jesse Daniel Brown and Rayssa Chiqueto. Clasy X3 implementation and independent evaluation are preserved by reference to their [source repository](https://github.com/JesseBrown1980/Clasy-repo-x-3-). The two SVG images are illustrative original artwork, not evidence.
