# Measured results — 2026-10-23

Rust `1.81.0 (eeb90cda1 2024-09-04)` and Cargo `1.81.0`, Linux/WSL, offline builds. The final formatting gate, all-target Clippy `-D warnings`, release build and **73 tests** pass with zero failures. Logs are in [logs/x3](logs/x3/).

## Stored 300 GB Host8 plane

The existing canonical packet specifies 375 sectors, 100,000,000 rooms/sector, 37,500,000,000 rooms, 8 bytes/room and ground room 18,750,000,000. The new reader uses that plane's seed `518bbb472a9db786`, not the earlier 100 GB plane's seed.

Each arm opens every sector read-only, requires the expected sector byte length, and reads nine disjoint windows spanning its first through last room. Every word is verified against the canonical invertible identity codec before it enters the unchanged upstream state projection. Each consecutive triple of stored stubs supplies one 81-coordinate model ensemble. The states are projections, not a claim that model amplitudes losslessly encode every original stub bit.

| Measured field | 81-room-width arm | 243-room-width arm |
|---|---:|---:|
| Physical sectors checked | 375 | 375 |
| Windows read | 3,375 | 3,375 |
| Stored-room reads | 273,375 | 820,125 |
| Payload bytes read | 2,187,000 | 6,561,000 |
| 81-mode ensembles | 91,125 | 273,375 |
| Checked transitions | 273,375 | 820,125 |
| State inverse-transform bundles | 91,125 | 273,375 |
| Transfer inverse-transform bundles | 91,125 | 273,375 |
| Identity/invariant failures | 0 / 0 | 0 / 0 |
| Coverage complete | yes | yes |
| Elapsed milliseconds | 32,141 | 32,951 |

Across the two arms: **1,093,500 room reads**, **364,500 ensembles**, and **1,093,500 checked transitions**. These are per-arm execution totals, not a deduplicated cross-arm unique-room count. They do not assert a full scan of 37.5 billion rooms. Initial-state and initial-transfer inverse bundles are counted per ensemble; the three subsequent steps get independent coordinate and invariant checks, not three extra NTT bundles.

Receipts: [ROOMS81.hbp](receipts/x3/ROOMS81.hbp), [ROOMS243.hbp](receipts/x3/ROOMS243.hbp).

## Existing sector schedule

The exact integer-only `oscillator.rs` and `trit.rs` modules are reused unchanged. The existing levels have 3/27/243 sectors and 45/405/3,645 local keys. All **4,095** keys pass the new shell-audit callbacks, comprising **12,285** additional transitions. Incomplete barriers are rejected, and local inverse keys recover their inputs. The 243-level's next-sector count 2,187 is the source function's calculated recurrence, not a newly allocated room group.

This exercises the lattice contract; it does not claim the source experiment's 729 model-worker roles were launched as external services. [LATTICE.hbp](receipts/x3/LATTICE.hbp)

## Long-run and independent mathematics

The preserved release verifier freshly passed:

- 10,000 exact 81-point NTT round trips;
- 100,000 coordinate step matches;
- four trajectories of 1,000,999 steps each: **4,003,996 transitions**, zero violations;
- three nonzero-drift scalar sequences visiting all 1,000,999 residues, and one inviscid scalar sequence with one distinct total.

The inherited step predictor shares its nonlinear routine with the implementation. The new independent tests and disk-run oracle do not rely on that shared predictor. The 17 independent tests include all 81 basis vectors, a Horner-form DFT, signed-integer endpoint equations, 1,536 trajectory steps and a separately exhaustive scalar-phase census. [VERIFY81.hbp](receipts/x3/VERIFY81.hbp), [audit81.rs](tests/audit81.rs)

The 73 declarations comprise 11 canonical tribit tests, 8 upstream 27-mode tests, 10 upstream 81-mode tests, 17 independent mathematical tests, 10 unchanged lattice tests, 6 Host8 audit tests, 4 argument tests and 7 I/O/failure-exit tests.

## Failure controls

The preserved old sweep was run with `--rooms 4 --threads 1 --sector /dev/null --no-step`. It completed zero ensembles but emitted `VERIFIED_81_INVARIANTS_PASSED` and exited 0. This concrete harness issue does not invalidate the separate successful mathematical tests; it explains why complete-coverage accounting was added. [Raw old receipt](receipts/x3/OLD-SWEEP-NEGATIVE.hbp)

The new binary exits nonzero, emits `X3FAIL`, and emits no success result for a missing store, truncated sector and invalid options. Short reads, empty/out-of-range windows and fixed little-endian fixtures have explicit tests. [Negative-control receipt](receipts/x3/NEGATIVE.hbp)

## Commands and provenance

Complete reproduction:

```bash
bash scripts/x3-gate.sh /mnt/d/ASOLARIA-HOST8-ROOMS-300G
```

After adding the seven final failure-path tests, formatting, Clippy, all-target tests and the release build were repeated successfully. The runtime binaries' mathematical and disk algorithms were unchanged by those test-only additions. The full-period and disk receipts are retained from the completed gate; the final tests log contains all 73 declarations.

The canonical native repository viewer was run on the frozen Git HEAD before source inspection. It admitted 24 files, held Cargo.lock, and checked its transform and independent SHA identities. Its `KERNEL_BIJECTION_NOT_DATA` scope is preserved; it is not presented as a hash decoding arbitrary source bytes. [Viewer receipts](receipts/x3/source-view/)

The root `SHA256SUMS` covers publication content other than itself. Verify from the repository root with `sha256sum -c SHA256SUMS`. Individual HBP sidecars are relative to their containing directory. The packet index has root-relative paths. Git archive verification is performed before push.

## Authority and scope ledgers

The native test binaries make zero provider/MCP/GPU/external-model calls, zero runtime subprocess launches and zero writes to the room store. Evaluation orchestration outside those binaries uses the provided shell and owning-system MCP; its acknowledgments are recorded separately. The Rust tests use bounded temporary fixtures and subprocesses for negative controls; these are not production room writes.

No descendant-cube learning census is inferred from this harness's external-call counter. Existing exposure, learned-relation, recurrent-transition and lineage ledgers remain separate from external service calls.

The operator confirms existing 81/243 room groups. This run probes those requested widths but does not verify the Omega/OmniHotel allocator binding or group-aligned ownership: probe offsets are explicit read-coverage positions. The owning request is acknowledged; no substantive binding answer was obtained for this evaluation. Existing architecture is preserved, and no allocator is invented or changed.

The proven shell-total invariant is explained in [X3-PROOF.md](X3-PROOF.md). These measured tests do not certify the continuum PDE bridge or Clay Millennium problem. The operator's 300 GB plane and the broader ecosystem are not reduced to this harness's sampled coverage.
