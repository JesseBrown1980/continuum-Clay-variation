# Exact Taylor–Green → Clay 81-register observable bridge

**Three consecutive time steps · three hash-linked leaves · independently audited**

**Verdict (2026-11-04):** Exact 81-observable conversion DERIVED. This mapping does **not** intertwine the two nonlinear time-evolution laws on the registered Taylor–Green vortex. Physical continuum PDE bridge remains OPEN.

## Mathematical definition

For the Taylor–Green periodic 24³ grid, let the 27 channel occupancies at cell x be n_i(x) in {0,1,2} and channel velocity components be c_{i,a} in {-1,0,1}. Partition the 24³ grid into 27 disjoint blocks B_b of size 8³. Map each block's three signed momentum sums into three Clay finite-field registers:

$$
J_{b,a}=\sum_{x\in B_b}\sum_{i=0}^{26}n_i(x)c_{i,a}\in\mathbb Z,\qquad
\Phi(n)_{3b+a}=J_{b,a}\pmod P,\quad P=1{,}000{,}999.
$$

There are 27 × 3 = **81** registers. Each block has 512 cells. There are nine positive and nine negative velocity channels on each axis, each occupied by at most two particles. Thus |j_a(x)|≤18 and |J_{b,a}|≤9,216. The centered representative in the prime field exactly recovers each **signed block momentum** (9,216 is less than P/2). The conversion is invertible for these **81 selected observables**, not for the entire Taylor–Green microscopic state.

The global sum correspondence is exact:

$$
Q(\Phi(n))=\sum_{b,a}J_{b,a}\equiv P_x(n)+P_y(n)+P_z(n)\pmod P.
$$

Unforced Taylor–Green gates and periodic streaming preserve each global momentum component. The Clay/Clasy shell model at viscosity parameter zero preserves its Q sum. **Only this scalar invariant is common**; it does not imply componentwise agreement of the dynamical updates.

**Explicit failure of full-state inversion:** Occupying a rest channel with velocity (0,0,0) changes density while leaving all 81 block momenta unchanged. The test retains two such distinct states with equal projection.

## Experiment: three leaves to observe movement

Inputs are from pinned [Taylor–Green commit ee71e0ee](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/commit/ee71e0eea7fd61bfe5a45ebf2f590311742161f7). The test replays the actual 288 D3Q27 gate lookup tables and the exact spatial streaming, then evaluates the projected registers. It compares with the **real Clay Rust library** ShellModel81.step() at ν=0 for each previous state.

All three arms—TGV, shuffled control SCR, and stationary NULL—are recorded after **t=1, t=2, and t=3**. The TGV observations matched the previously committed Taylor–Green tick-0..3 E2, KJ, WJ and coherent-amplitude measurements.

| Step | TGV trits changed | TGV registers changed | Local Clay-vs-TGV component differences | KJ | WJ | Amplitude A |
|---|---:|---:|---:|---:|---:|---:|
| 0 | — | — | — | 106,496 | 108,032 | 35,328 |
| 1 | 169,912 | 81/81 | **81/81** | 382,752 | 1,225,664 | 37,128 |
| 2 | 237,204 | 80/81 | **81/81** | 537,904 | 1,876,856 | 34,600 |
| 3 | 244,184 | 80/81 | **81/81** | 573,144 | 2,038,608 | 33,688 |

Unforced TGV and SCR maintain global momentum (0,0,0), mass 373,248, and twice channel energy E2=746,496. **NULL remains stationary:** zero changed trits, zero changed registers and zero Clay comparison defects at all three steps.

The local nonlinear commutation test is

$$
\Phi(T_t(n_{t-1}))\stackrel{?}{=}C_0(\Phi(n_{t-1})).
$$

Here T_t is the actual Taylor–Green gate scheduler followed by streaming, and C_0 is one Clay update at ν=0. **It fails at all 81 components at each tested TGV step.** A separately iterated Clay trajectory also differs. This is a *registered negative result* and does not invalidate the proven exact projection or shared total momentum.

The observed large WJ rise is also found in the shuffled control, consistent with the prior [pre-registered lattice noise test](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved/tree/main/results/2026-10-30-noise-test). KJ and WJ are Taylor–Green harness diagnostics, not Clay energy and enstrophy.

## The three SHA-256-linked leaves

Each leaf contains three arm records, 81 signed block momenta, 81 reduced register values, global conserved totals, changed-trit/register counts, and local and sequential model defects. Each leaf header pins its predecessor.

| Artifact | SHA-256 | Parent |
|---|---|---|
| LEAF-1.hbp | ee5036f22e94990de1992e4649302d603aa5a2891222b9333a512959c95b40be | GENESIS |
| LEAF-2.hbp | 67c1ceb79d45761b68835910e0a05c76564785b77b487bff3fe6d5a49901b11e | LEAF-1 |
| LEAF-3.hbp | c638926523a7b45e44e1396320a4f3a640c17633716e65824351a8a543819c9f | LEAF-2 |

[GitHub Actions workflow](https://github.com/JesseBrown1980/continuum-Clay-variation/actions/workflows/taylor-clay-3leaf.yml) publishes a downloadable artifact named **taylor-clay-3leaf**, containing three HBP leaves, LEAVES.hbi, SHA256SUMS, RUN.hbp and NEGATIVE.hbp.

The separately coded [receipt auditor](verify_receipts.py) validates the complete hash chain, exact 81-residue conversions, original measures, conservation, stationary NULL and noncommutation. The runner also deliberately corrupts trit input, gate input and a leaf receipt, requiring fail-closed rejection.

## Prediction-first reproduction

Two [HBP predictions](PREDICTION.hbp) and [the three-leaf addendum](PREDICTION-3-LEAVES.hbp) were filed before the executable was run. The Taylor–Green SHA-pinned source inputs are:

- TGV initial state: eeaa2c52e244cdc4cfdc4b1f6986539ef3a3bd64d4b908b9c1b747ea78742434
- NULL initial state: ab79efe04d4d0d13b5edad664bc235040f51fe94fd6230f955b833806825d0ae
- 288 lookup-table gates: a7fca71ac8a85fd3acbe4188e9c246e98e4beeeaa459f3e77d40c0342aa4bb12

On Linux/WSL with Rust 1.81 and Python 3, check out Taylor–Green at the pinned commit in bridge/taylor-green/tg-source, then run:

~~~bash
bash bridge/taylor-green/run.sh
python3 bridge/taylor-green/verify_receipts.py bridge/taylor-green/out
~~~

The Rust implementation is offline, integer-only, and compiles against the actual local Clay crate. The three HBP leaves are generated under bridge/taylor-green/out; the shell process records SHA-256 digests. The [CI workflow](../../.github/workflows/taylor-clay-3leaf.yml) performs the pinned checkout and the whole replay.

## Verdict and continuation

| Question | Result |
|---|---|
| Exact 81 signed momentum observables | **DERIVED / VERIFIED** |
| Centered inverse for those 81 observables | **DERIVED / VERIFIED** |
| Global momentum ↔ Clay total Q | **DERIVED / VERIFIED** |
| Pinned Taylor–Green gate replay through t=3 | **PASS** |
| Three-leaf SHA-256 verification and negative controls | **PASS** |
| Full microscopic Taylor–Green state inversion | **REFUTED for this map** |
| Dynamic commutation of the nonlinear updates | **REFUTED on the tested TGV trajectory** |
| Another dynamical intertwiner, faithful reconstruction, or physical PDE correspondence | **OPEN** |

The next task is a projection with explicitly registered scale, frequency and aliasing conventions, a common normalization for physical observables, and a viscosity mechanism that matches across the models. It needs quantitative approximation bounds on increasing grids; exact finite-field coordinates alone cannot imply a continuum Navier–Stokes solution.

Related repositories: [Clay](https://github.com/JesseBrown1980/continuum-Clay-variation) · [Clasy X3](https://github.com/JesseBrown1980/Clasy-repo-x-3-) · [Taylor–Green](https://github.com/JesseBrown1980/Taylor_green_regularity-resolved-and-solved).
