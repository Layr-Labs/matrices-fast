# 0098: bounded structural terminal candidates and allocation reuse

Date: 2026-09-07. Effort: max.

Final local result: **0.806370063938585** versus untouched ef9ec4c
**0.806787941749786**, 17 improved matrices, zero regressions, 283 unchanged.
All 300 matrices passed the official sandboxed local harness. The submitted
version retains the canonical METIS seed correction and omits the late donor
continuation. Remote submission `d21ea6f6-b053-4320-b038-0bfcf306f202`
subsequently **FAILED without a score** (recorded source `80f4fe1`). The CLI
exposed no error category or case. The cause is unknown: a local cap pass is
not a hidden-validation certificate, and no leaderboard gain is claimed.

## Context and baseline

This work builds on promoted source
`ef9ec4cb00faf0a32d8911b6cf3c0aae17e1eab7`, submission `6fa0167` by
companygardener. The upstream hidden score observed before submission was
0.851146. That is a different corpus from the public development set; the
local numbers below are not a predicted hidden score. The schema-v1 manifest
requires a one-basis-point improvement, not the ten-basis-point threshold of
the older SSI challenge. No track selection is involved.

The new matrices-fast repository was cloned with Git LFS available. Work was
isolated in Git worktrees, with an untouched worktree used for the fresh
upstream control. Both source and generated build artifacts were checked
before testing. The public corpus contains 300 patterns and has SHA-256
`faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`.
The local machine is Darwin arm64, with Rust/Cargo 1.98.0, GNU GCC 16.2.0
installed, git-lfs 3.8.0, cargo-deny 0.20.2, and Yukon v2026.09.05-2.

The inherited implementation is a scored portfolio, not a single AMD call.
It includes the exact grader AMD anchor, minimum-degree and minimum-fill
variants, partitioning methods, relabelled restarts, structural core
reductions, and bounded elimination-game and completion refinements. The
score is the weighted arithmetic mean of three within-bucket geometric
means of exact predicted flop ratios, with weights 0.30/0.30/0.40.
The objective for one ordering is the sum of squared symbolic column counts.

## Hypothesis and implementation

The hypothesis was that structural perturbations of the completed incumbent
could reach orderings missing from the large existing portfolio. Preserving
the early portfolio and its runner-up ledger was important: an extra early
candidate can change subsequent search trajectories and consume another
candidate's work allowance. Strict acceptance at one phase is not a proof
that moving that phase earlier improves the entire pipeline.

Only `src/ordering/` is changed. No dependency was added or upgraded. New
bookkeeping, graph construction, ranking, and scratch reuse use the Rust
standard library and call the already-present ordering/scoring interfaces.
The trusted harness, corpus, manifest templates, and scoring library remain
unchanged.

### STRIP: defer a small structural prefix

`strip.rs` constructs a family by removing a small ranked vertex prefix,
ordering the induced remainder with AMD, and appending the removed vertices
in ascending `(original degree, vertex ID)` order. The rankings are original
degree; degeneracy core number with original degree and one-hop degree mass
as tie-breakers; and the anchor AMD factor's exact symbolic column counts.
The fixed degree prefix sizes are 96/64/48/24/6. Core ranking uses 64/24/6,
and the anchor-count ranking supplies a 64-vertex candidate when possible.
Invalid or oversized prefixes are skipped.

The graph family is gated by `n < 30000`, `nnz < 130000`, and `nnz < 25*n`.
These bounds limit work and allocation; they do not identify input matrices.
An incremental builder starts with the largest removal and restores vertices
for successively smaller removals. The emitted local CSC numbering is
identical to independently rebuilding each induced subgraph. Tests compare
both CSC arrays and the resulting ordering bytes with the naive construction.

This stage runs after the inherited transplant and before MINL. A strict
STRIP win may receive up to four existing MCS extraction rounds. A stalled
round stops, factor nonzeros are capped at one million, and rounds share a
2.5-million-unit `n + nnz + factor_nnz` ledger. The ledger is a structural
work bound, not a guarantee of a particular wall-clock duration.

### TELOS: perturb the highest-cost symbolic columns

`telos.rs` ranks the current ordering's vertices by exact symbolic column
counts, then tries induced-subgraph reorderings and simple tail splices from
one immutable snapshot. The induced suborders use existing AMD, no-dense AMD,
and AMF-at-alpha-5 implementations. Candidate results are exact-scored;
ties retain the incumbent.

The entry gate is `16 <= n <= 30000` and `0 < nnz < 400000`. Below 150000
input nonzeros, at most two rounds use a fixed priority list with
`clamp(700000/nnz, 3, 16)` induced-suborder candidates and 15 fixed peel
sizes. The denser tier uses one round, two suborders, and three peel sizes.
At most two existing runner-up orderings receive a smaller one-round pass
inside a tighter input-work/maximum-degree gate. The same immutable input
and rankings are shared with the existing deterministic parallel engine.

### Rejected experiment: separate late metric donors

An intermediate version gave the original seven relabel-metric families a bounded continuation from
their existing 120000/nnz schedule to 240000/nnz, retaining the six-pass cap.
The additional tickets continue the existing arithmetic seed sequence:
`30000 + family_index*1000 + pass_index`. No seed sweep or favorable-seed
selection was used for this continuation. Later-added metric families keep
their inherited treatment.

Generation was deferred until the previous terminal stages had completed.
The experimental score-unique top-eight donor ledger was separate from the inherited
runner-up ledger, so a continuation donor cannot evict an earlier seed or
consume that seed's allowance. Its alternate-PEO walk uses a separate
four-million-unit work ledger and the existing bounded transplant. Admission
requires `n <= 50000` and the existing sparse metric gate. Any replacement
is exact-scored against the fully finished incumbent. This part bought only
a small marginal development gain. After an official local cap failure,
focused phase measurements put its cost at 0.13-0.16 seconds on two
cap-sensitive cases where it produced no gain. The entire continuation and
its additional ledger were removed. This submission does not increase the
inherited metric-relabel budget or retain new donor seeds.

### Terminal MCS tie alternatives

The last phase adds four maximum-cardinality-search extractions from the
current chordal completion: original vertex ID and completed-graph degree,
each in forward and reverse rank order. They are kept outside the earlier
incumbent-dependent chains and paid for once, inside the existing
`n <= 30000`, `nnz <= 180000`, and one-million-factor-nonzero limits.
The completed graph is reconstructed once, each result is a valid perfect
elimination order of that completion, and the original input is exact-scored
before accepting a strict improvement. Exhaustive tests cover every labelled
graph and every input permutation through five vertices, including independent
PEO checks and exact flop non-regression.

### Allocation and inherited-seed cleanup

`minl.rs` uses the sorted, unique original CSC column directly for original-edge
membership. It no longer clones and sorts a second complete original adjacency
list. Its clique ordering scratch vector is cleared and reused. In `mod.rs`,
the sorted, score-unique runner-up ledger checks whether an entry can survive
before cloning an entire permutation. These are output-preserving changes;
their full-corpus exact flop counts were unchanged in the allocation-only
comparison. `scoring_ws.rs` exposes the most recent column counts read-only
so the anchor ranking can be captured before workspace reuse.

The final audit also found an inherited METIS seed-21 candidate. The historical
probe compared it with seed 2 and retained 21. The submitted policy replaces
it with the ordinary next seed, 2, after the library's default seed 1. This
correction is retained independently of whether it helps the development
score. Fixed deterministic randomness is allowed; selecting a known-favorable
random basin is not the intended methodology. Matrix names below are reporting
labels only, never input to the production ordering policy. There are no new
matrix fingerprints, hard-coded output permutations, external reads, or
clock-dependent stopping conditions.

## Reproduction and validation

From each benchmark worktree:

```sh
yukon setup
yukon run
```

The manifest invokes the supplied sandboxed candidate build followed by the
trusted parent. The parent checks purity/licenses, invokes each matrix twice,
checks deterministic bijections, enforces the two-second local worker cap,
and recomputes symbolic flops. The reported `(capped)` rows passed with the
watchdog enabled; that marker is not a timeout. Integer per-matrix counts from
the table were used to recompute the weighted score at full precision.

Candidate tests were also run with the equivalent macOS build sandbox:
network denied, source read-only, writes confined to target/cache/temp roots,
using `cargo test --release -p ssi-candidate-worker --offline --locked`.
The structural stack passed 77 active tests, with zero failures and 30 ignored
research probes. The final METIS seed correction and removal of the donor
experiment were then tested by a fresh official 300-pattern run, which passed.

The fresh untouched upstream control and the structural stack before the
seed correction both passed all 300 matrices:

| Variant | Exact weighted flop ratio | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| Untouched ef9ec4c | 0.806787941749786 | 0.889737453581 | 0.845471807351 | 0.715562908676 |
| Structural stack, before seed correction | 0.806348775926643 | 0.889728153684 | 0.845190654604 | 0.714682833601 |

This is 4.39166 absolute score basis points, or 5.44339 relative basis points,
with 20 improved rows, zero regressions, and 280 unchanged rows. The aggregate
is empirical; the pre-MINL placement does not imply universal pointwise
dominance of the complete pipeline. These are intermediate experimental
results, not the final submitted score.

Representative exact flop changes before the seed correction:

| Public dev label | Upstream | Structural stack |
|---|---:|---:|
| maxcsp-ehi-85-297-71 | 1111065177 | 1072351213 |
| crudeoil_lee4_10 | 199013967 | 198559373 |
| crudeoil_lee4_06 | 43251755 | 43155507 |
| gabriel09 | 27484225 | 26932633 |
| gams05 | 3481397874 | 3479070222 |
| popdynm200 | 2507726 | 2446837 |

Other improved rows were mpbp_48, chimera_mgw-c16-2031-01,
crudeoil_pooling_dt2, nd_netgen-2000-3-4-b-a-ns_7, procurement1large,
torsion50, powerflow0300p, pooling_sppa0pq, crudeoil_lee1_07,
chp_shorttermplan2d, crudeoil_lee2_06, qspp_0_11_0_1_10_1,
crudeoil_lee4_09, and mpbp_35.

## Final submitted candidate

After removing the donor expansion and keeping canonical METIS seed 2, the
fresh official `yukon run` passed all 300 matrices, including the case that
had previously hit the watchdog. Its exact per-bucket flop ratios are
0.889699744059344 / 0.845193031752257 / 0.714755577987762, giving weighted
score **0.806370063938585**. The emitted six-decimal score is 0.806370 and
the fill tiebreak is 0.930385. Relative to the fresh ef9ec4c control this is
**4.17878 absolute score basis points / 5.17952 relative basis points**.

There are 17 improved rows, zero regressions, and 283 unchanged rows.
Compared with the intermediate table above, nd_netgen-2000-3-4-b-a-ns_7,
torsion50, powerflow0300p and pooling_sppa0pq return to the upstream counts.
`chp_shorttermplan2d` finishes at 2111293 and `mpbp_35` at 980034, both still
strictly below upstream. `multiplants_mtg1b` improves from 152043 to 151331
with the canonical seed correction. The other reported gains are unchanged.

The final run is evidence for this local corpus and machine only. It does
not establish the hidden score, promotion, or a universal two-second bound.

## Ablations and rejected directions

The first seed-corrected donor version failed the official local time cap at
`mpbp_15` after 254 completed rows. A focused three-repeat phase profile then
measured minimum total times of 0.6292 seconds there, 0.8774 on `mpbp_48`,
and 0.7812 on `crudeoil_lee4_10`. The donor phase alone took 0.1339 and
0.1428 seconds on the first two rows, with no score change. This does not
prove the cause of the failed watchdog run; it identifies removable work.
The candidate was reduced before retrying, rather than treating the
successful focused repeats as clearance to submit the failed version.

The preceding `fcb74a7` base was freshly measured at
0.807259159160610. The structural stack before donor continuation measured
approximately 0.806841, unchanged by the allocation-only rewrite; donor
continuation measured 0.806809733882827. These are same-base ablations, not
numbers to compare directly with the newer ef9ec4c control.

Several follow-on probes are deliberately excluded:

- Linear partition-refinement LexBFS with four ID/degree directions passed
  exhaustive correctness tests but changed none of the 300 exact flop counts.
- Expanding MCS to original-degree, fill-surplus, and current-column-count
  ranks with an earned four-round chain measured 0.806288305962967 on the
  pre-correction stack: 11 wins, zero losses, about 0.75 relative dev bip.
  It is a separate follow-on, not part of this submission or its cap claim.
- Composite rank keys and FIFO/LIFO recency variants were mostly dominated
  by that simpler rank chain. Their novel marginal benefit was about 0.01
  full-dev bip as projected from a focused subset. Heap-based MCS also added
  substantial runtime. They were rejected without a full-corpus claim.
- A count/fill-ranked second MINL scan had only a small focused-subset gain
  and added approximately 0.09-0.48 seconds in the loaded direct probes.
  It is not shipped; those probes were not an official full-corpus result.
- Uniform metric/relabel grids and additional donor families produced zero
  or concentrated low-value gains. Enlarging work allowances alone was not
  sufficient evidence to retain them.

These negative results are evidence about these implementations on this
public corpus, not evidence that the algorithm families are universally
unhelpful. Test-only phase instrumentation and host contention made some
direct timing probes exceed production harness times. No single explanation
is assumed for that difference. The official sandboxed run is the local cap
gate; hidden validation, different hardware, and the four-GiB graded address
space cap remain separate checks. Peak RSS was not measured here.

## Learning and next steps

Preserving an incumbent-dependent pipeline requires preserving its inputs,
not merely appending entries to an early best-of list. Independent late donor
storage prevented eviction and budget interactions in the experiment, but
did not justify its added time. Exact terminal acceptance does not erase the
cost of constructing and scoring candidates that never win.

The most useful follow-ons are a bounded version of the wider structural MCS
rank chain and component-wise mixing of already-scored donor orders. The
latter could exploit separability of disconnected components without new
random trajectories. It remains an unimplemented hypothesis in this checkpoint.
Neither development improvement nor successful local validation establishes
leaderboard promotion; the remote result must be inspected separately.

Related pages: [0097](0097-descent-fill-gate-and-restored-sub10k-lotteries.md),
[0095 MINL](0095-terminal-completion-lattice-descent.md),
[portfolio architecture](../techniques/best-of-portfolio.md).
