# Companion research track: Taylor–Green D3Q27 and Clay/Clasy X3

**Status: repository cross-links established; a mathematical state-to-state bridge is OPEN.**  
Created 2026-11-04. The original models, data and pre-registered failures are preserved.

| Track | Repository | State and governing update | Established scope |
|---|---|---|---|
| Clay | [continuum-Clay-variation](https://github.com/JesseBrown1980/continuum-Clay-variation) | 81 finite-field coordinates in ℱ_P, P=1,000,999; `u'_n=u_n+NL_n(u)-nu*9^n (mod P)` | All-time compensated modular sum invariant and invertible 81-point NTT |
| Clasy X3 | [Clasy-repo-x-3-](https://github.com/JesseBrown1980/Clasy-repo-x-3-) | Independent integer checks of the same 81-mode map; 81/243 *room-read widths* and 3/27/243 transition-lattice contract | Independent oracles, 73 recorded passing tests, sampled Host8 audit and negative controls |
| Taylor–Green | [Taylor_green_regularity-resolved-and-solved](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved) | Periodic D3Q27 lattice gas, 27 per-cell occupancy channels (0,1,2), 288 reversible collision gates and streaming | Exact finite-grid occupancy and collision conservation; measured vortex, noise, forcing and transport experiments |

**Pinned source snapshots when the link was authored:** Clay main `01c07efef3bcb7ce0d94234994432250c4094774`; Taylor–Green main `ee71e0eea7fd61bfe5a45ebf2f590311742161f7`. Refer to current main for newer changes.

## What genuinely connects

Both are finite, deterministic, integer-arithmetic research models involving ternary representations. They provide different controlled settings for testing invariants and modeling assumptions. Their shared research questions concern finite-state conservation, nonlinear transfer, discretization effects, resolution dependence and the gap between discrete models and physical PDEs.

There is **no demonstrated conjugacy, state embedding, exact spectral projection, PDE reconstruction, common energy functional, or transfer of regularity theorems** between them. The number 27 occurs in different roles: Taylor–Green uses 27 velocity channels *per spatial cell*; Clay uses 81 algebraic shell amplitudes and a separate 3/27/243 transition-sector representation.

## Important non-equivalences and countertests

1. **Zero initial state.** Taylor–Green's equilibrium NULL control remains stationary, while Clay's `u=0, nu=1` maps to `u'_n=-9^n (mod P)`. Thus the updates cannot be identified without a separately defined, verified transformation.
2. **Viscosity.** Clay's term `-nu*9^n` is an additive forcing/drift independent of amplitude. Taylor–Green's effective viscosity is inferred from its collision/streaming dynamics; the October 30 calibration reports ~0.4397 and ~0.4258 in different finite-grid runs. These parameters are **not interchangeable**.
3. **Conserved quantities.** Clay proves an affine law for `Q=sum(u_n) mod P` by the deliberately chosen endpoint closure, not physical kinetic energy conservation. Taylor–Green's microscopic occupancy/collision accounting conserves its integer mass, momentum and channel energy under unforced evolution; forced arms explicitly inject momentum. None of these is automatically Clay's `Q`.
4. **Noise controls.** The [Taylor–Green pre-registered noise test](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-noise-test) found the large enstrophy rise agrees with a shuffled no-vortex control within 10% on 476/476 ticks. Its coherent vortex amplitude decayed. Therefore the earlier README attribution of *all* enstrophy increase to coherent vortex stretching should not be propagated to Clay as an established causal result.
5. **Finite-grid regularity.** Taylor–Green's occupancy bound is valid at every fixed lattice spacing, but its discrete derivative bound grows like `1/dx`. It does not supply a spacing-uniform continuum PDE bound. Clay's modular residues likewise do not bound unreduced continuum norms.

The Taylor–Green [collapsing-vortex test](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-cutoff) has registered negative as well as positive results. Preserve its failed `A1` control and the cutoff comparisons verbatim.

## Proposed verification-first interface (NOT YET IMPLEMENTED)

Before claiming a cross-model numerical or analytic bridge:

1. **Specify observables** on both sides. For Taylor–Green this could begin with `rho(x)`, `j(x)`, and `u(x)=j(x)/rho(x)` when `rho>0`, plus a fixed spatial domain, cell size and time-step convention. For Clay, specify which real-valued observable is reconstructed from finite-field representatives, and how reduction modulo P affects it.
2. **Specify the projection or reconstruction** (spatial-to-shell, inverse if proposed) with exact coordinate definitions, normalization, aliasing rules, and a meaningful frequency cutoff. `k_n=3^n` for 81 modes is not directly resolved on the Taylor–Green 24^3, 48^3, or 96^3 grids.
3. **Register predictions before running**, including the zero-state disagreement and shuffled-noise control, and provide negative controls that reject an incorrectly normalized projection.
4. **Certify consistency and errors** with independent integer/rational or interval arithmetic and preserved sample/receipt hashes. Separate exact arithmetic identities, observed finite-resolution agreement, and any limiting theorem.
5. **Hold physical/continuum claims OPEN** pending divergence-free reconstruction, correct viscosity, PDE consistency, convergence and resolution-uniform physical norm estimates.

## Evidence and reproducibility

- Clay: [corrected 81-mode proof](CLASY-X3-PROOF.md), [README](../README.md), [portable CI script](../scripts/x3-ci.sh).
- Clasy X3: [results and known limitations](https://github.com/JesseBrown1980/Clasy-repo-x-3-/blob/main/RESULTS.md).
- Taylor–Green: [noise](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-noise-test), [forced-pressure](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-forced-pressure), [cutoff](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-cutoff), [calibration](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-calibration).

**Interpretation:** This document links the two codebases and their evidence, not their mathematical state spaces. It records an honest starting contract for a future independently testable interoperability layer.
