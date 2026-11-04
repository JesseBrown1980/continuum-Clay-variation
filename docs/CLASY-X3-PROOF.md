# Exact theorem and the continuum bridge

## 1. The implemented mathematical object

The state is `u ∈ F_P^81`, `P=1,000,999`. Time is a nonnegative integer.
The implemented update is

`F_n(u) = u_n + NL_n(u) - (nu mod P) (9^n mod P) (mod P)`.

For `1≤n≤79`, `NL_n=u_n(u_(n+1)-u_(n-1)) (mod P)`. Let `S` be the canonical residue of the interior sum, in `0..P-1`. The endpoint values are

`NL_0=-floor(S/2)`, `NL_80=-S+floor(S/2)` (mod P).

Ordinary integer floor is intentional; it is not finite-field division by two.
The implementation's term called dissipation is an additive coordinate-dependent term, independent of `u_n`. No claim of amplitude damping follows from its name.

## 2. Universal transfer cancellation

Summing the endpoint definitions with the interior gives

`-floor(S/2) + S - S + floor(S/2) = 0`.

Thus `sum NL_n=0` for every state. This is an exact boundary-closure identity, not a statistical observation. Interior products also telescope to `u_79*u_80-u_0*u_1`, which the independent tests check.

## 3. The exact all-time solution for the total

Define `Q(u)=sum u_n (mod P)`. Iterating `9^n` and summing gives
`D=sum_(n=0)^80 9^n=900,647 (mod P)`, with `8D=9^81-1 (mod P)`.
The zero-transfer sum yields

`Q(F(u))=Q(u)-(nu mod P)D (mod P)`.

Induction proves, for every initial state and every `t≥0`,

`Q(u_t)=Q(u_0)-t*nu*D (mod P)`

and therefore `I(t,u_t)=Q(u_t)+t*nu*D=Q(u_0) (mod P)`.

This is the exact all-time invariant theorem validated by the implementation tests. It does not solve every coordinate in closed form. Since `P` is prime and `D≠0`, the scalar sequence visits every residue once per period `P` exactly when `nu mod P≠0`. For `nu mod P=0`, its period is 1. Positive `nu=P` is an important edge case.

## 4. The transform and the 243 architecture

`W=30,722` has order 81 in `F_P*`. The standard finite geometric-sum identity makes the forward and inverse 81-point NTT mutually inverse. At transform index zero, all weights are one, so `prism81(NL)[0]=sum NL=0`.

Independent tests verify primality, generator order, transform constants, all basis vectors, and arbitrary full spectra rather than checking DC alone.

`243` is the existing sector-lattice scale and an operator-confirmed room-group width, not automatically a new transform width. In this field `(P-1)%243=81`; the existing `W` still has order81, even though `W^243=1`. The test does not mislabel three 81-mode transforms as one primitive 243-point NTT.

The unchanged lattice modules retain all five representation edges and three local tagged directions at each sector. Its declared levels `3→27→243` have `45/405/3645` keys. New shell callbacks exercise that schedule; they are not service invocations of Fischer/MTP/HRM or an allocator deployment.

## 5. What is and is not inverted

NTT inversion, room codec inversion, tagged local direction inversion and nonlinear dynamical inversion are separate predicates.

The tests retain a concrete dynamical witness: let `w_1=1`, `w_2=P-1`, and all other coordinates be zero. Then `w+NL(w)=0`, so `F_nu(w)=F_nu(0)` for every viscosity. Full NTT invertibility does not make this update injective. Likewise constant scalar total does not force a constant full state.

## 6. Unreduced norms and continuum regularity

For fixed `P` and 81 modes, centered representatives satisfy `|u_n|≤(P-1)/2`. This gives the ordinary integer bounds

`sum u_n^2 ≤ 81*((P-1)/2)^2`,

`sum 9^n*u_n^2 ≤ ((P-1)/2)^2*(9^81-1)/8`.

These statements do not reduce the norms modulo `P`, but depend on the fixed modulus and resolution. They are not uniform continuum estimates. The runtime uses reduced dissipation weights; it does not evaluate `9^80` as a physical unreduced damping coefficient.

Linear transfer balance is not quadratic energy cancellation: for `u_0=u_1=1` and the remaining coordinates zero, `sum u_n*NL_n=(P-1)/2≠0 (mod P)`. Starting from zero with `nu=1` produces `u'_0=P-1` and nonzero values in all coordinates. These are properties of the implemented map and must be handled by any proposed physical reconstruction.

To establish the continuum Navier–Stokes result, the system bridge must specify a divergence-free reconstruction, its domain and forcing/viscosity interpretation, consistency with the PDE, convergence as resolution changes, and sufficient unreduced norm/continuation control. Larger finite room sweeps improve implementation coverage; they do not substitute for those quantified bridge obligations.

The upstream PROOF.md itself lists reconstruction, uniform norm control and strong convergence as requirements. This evaluation preserves the successful finite-field theorem and does not silently promote finite residue bounds to continuum vorticity estimates. Owning-system queries seek the bridge and hotel binding; their acknowledgments are not mathematical ratification.
