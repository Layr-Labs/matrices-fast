# Restore the promoted `Game` buffer lifecycle: remove the last post-completion divergence

## Summary

This submission makes **no search change**. It removes the four buffered-image
changes that were the only functional difference between the current tree and the
promoted base `05fa99b`, restoring the working-image lifecycle that the promoted
tree used when it scored **0.840511** on a live hidden corpus.

The exact public score is unchanged: the full 300-row development probe reports
**SCORE = 0.790254** with **300/300 `COUNTS` records**, identical to the tree that
preceded this change. The release ordering suite is **127 passed / 0 failed / 56
ignored**.

This is deliberately a cap-margin submission, not a value submission. It does not
enlarge any structural gate, ledger, width, sweep count, component set, or
candidate pool, and it does not change the strict exact-flop acceptance rule.

## Why this is the right next step: the divergence is smaller than it looked

Four consecutive submissions on 2026-09-15 died at the same position with the same
error, and the error is a wall kill, not a memory kill:

| iteration | submission | PR | Benchmark time to cap |
|---|---|---|---:|
| iter77 | `b84237f` | #743 | 84.509 s |
| iter78 | `782a26d` | #744 | 84.681 s |
| iter79 | `26f00f8` | #745 | 84.360 s |
| iter80 | `65c2e9d` | #746 | **84.106 s** |

The failure string is identical in all four:

```
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

A four-endpoint band of 84.1–84.7 s across trees that differ by two search devices
means the kill is not ordered by tree cost. Two trees of this family **did**
complete a live corpus — the promoted base `f1782ef` at 0.840511 and this lane's
resource-law tree `0e1edc92` at 0.840512 (PR #722, 14 min 02 s). That makes the
post-completion delta the highest-value thing to audit, so I fetched the promoted
base's ordering sources and diffed them against the submitted tree.

The result is that the two trees agree on far more than the handoff records:

| component | promoted `05fa99b` | submitted iter80 | same? |
|---|---|---|---|
| `rgreedy::MAX_N` | `1 << 17` | `1 << 17` | yes |
| sparse `Pristine` representation | present | present | yes |
| `SPARSE_PRISTINE_MIN_WORDS` | 20 000 000 | 20 000 000 | yes |
| `Game::new_sparse` | present | present | yes |
| one-GiB image envelope | `1 << 27` words | `1 << 27` words | yes |
| `nnz <= 16n` density law | present | present | yes |
| `PRODUCTION_EXCHANGE_LEDGER` | 2 147 483 648 | 2 147 483 648 | yes |
| `mod.rs` non-comment lines | — | 37 changed | all `#[cfg(test)]` seam defaults |
| `window_dp.rs` | — | 15 hunks | test instrumentation only |

`window_dp.rs` carries exactly one production change, `PRODUCTION_XCH_ALLOC: 0 -> 1`,
which was measured as a single-mover, wall-neutral change and which the promoted
tree's own successor also carries. Every other `mod.rs` difference is a test-only
seam default, with the single exception of the `peo_alt_seeds()` accessor, which
returns the promoted constant 32/8 in a non-test build.

That leaves **four functional changes**, all in `rgreedy.rs`, and all of them in the
working-image lifetime. They are what this submission removes.

## Exact production constants after this change

```
rgreedy::MAX_N                          = 1 << 17            (131 072)
SPARSE_PRISTINE_MIN_WORDS               = 20 000 000
ADJ_POOL_MAX_WORDS                      = 20 000 000         (one 160 MB buffer per thread)
PRODUCTION_XCH_ALLOC                    = 1
PRODUCTION_EXCHANGE_IMAGE_WORDS         = 1 << 27
PRODUCTION_EXCHANGE_SPARSE_FACTOR       = 16
PRODUCTION_EXCHANGE_LEDGER              = 2 147 483 648      (2 GiB)
PRODUCTION_PEO_ROUNDS                   = 4
exchange window                         = width 12, sweeps 12, step 5
dense/hub twin                          = 8 / 4 / 3
PRODUCTION_SPAN_WINDOWS                 = 9 schedules, width 48/9/8/12/7/10/11/14/6
```

Terminal admission is unchanged and remains structural, never per-matrix:

```
past_anchor  &&  n >= 6  &&  n <= MAX_N  &&  nnz <= usize::MAX
             &&  n * ceil(n/64) <= 1 << 27
             &&  nnz <= 16n
             &&  max_deg <= n / 2
```

No identity gate, corpus lookup, clock, environment dependency, or
nondeterministic iteration source is introduced. The graded worker sees a
`cfg(not(test))` build in which every seam above compiles to the constant shown.

## What changed, precisely

For a graph of dimension `n` with `w = ceil(n/64)`, the exact-window `Game` holds a
mutable bitset of `n*w` words. The `Pristine` object is the immutable reset source.
Four lifetime behaviours were changed after the last completion, and all four are
reverted here:

1. **Sparse-image retention.** `Drop for Game` used a per-path ceiling: a sparse
   game kept a recycled image up to `1 << 27` words (1 GiB) alive in a thread-local
   pool. It now uses the promoted single `ADJ_POOL_MAX_WORDS = 20 000 000` (160 MB)
   ceiling for every game, so a large image is released to the allocator as before.
   This changes lifetime, not the number of simultaneously live images.

2. **Fresh sparse image.** `Game::new_sparse` now materializes its image into a
   fresh `alloc_parallel_zeroed_u64_vec`, releasing the pooled buffer it took. The
   first `reset` on the sparse path therefore cannot observe arbitrary recycled
   words; the reset contract no longer depends on a separate flag.

3. **No reset-first constructor.** `Pristine::game_reset_first`,
   `Game::new_reset_first_with_degrees` and the `initialize_adj` parameter of
   `Game::assemble` are removed. `Pristine::game()` calls `Game::new_with_degrees`,
   which fills the pooled image from the pristine bitset with `copy_from_slice`
   before any reader can observe it. The test-only `SSI_XCH_EAGER_ADJ` seam is
   retired with the constructor it priced.

4. **No first-reset fast path.** The `sparse_adj_is_pristine` flag and its branch in
   `reset` are removed. Every sparse `reset` clears and rebuilds from CSR, exactly
   as in the promoted tree.

Net effect: **−55 lines** in `rgreedy.rs` and one call site in `window_dp.rs`. No
search budget, admission gate, charge model, or acceptance rule changes. `reset`
still charges the identical `2*n*w + 8n` on every path, so every logical budget and
every search trajectory is unchanged by construction.

The previous behaviour remains reproducible from commits `0503544` (reset-first) and
`0696f9a` (sparse first-reset and buffer retention).

## Evidence: output invariance on the development corpus

The complete 300-row probe (`SSI_MARK_NOSCORE=1`, one binary, one session) reports
**SCORE = 0.790254** and emits **300/300 `COUNTS` records**. The four rows whose
records the sparse terminal path is known to move reproduce their exact flop counts:

| row | n | nnz | candidate flops (before → after) |
|---|---:|---:|---:|
| `nuclear104` | 39 098 | 257 806 | 78 332 024 → **78 332 024** |
| `transswitch2736spr` | 69 651 | 331 010 | 7 281 507 → **7 281 507** |
| `transswitch2383wpr` | 59 853 | 277 562 | 3 861 509 → **3 861 509** |
| `gams05` | 17 364 | 252 910 | 3 259 322 396 → **3 259 322 396** |

Their phase profiles are likewise unchanged through every marked stage
(`1.portfolio`, `9.reduce`, `17.final`), confirming that the removed fast path was
work with no reader rather than a behaviour the search depended on.

## Rejected: narrowing `MAX_N` back to 80 000

Before settling on the lifetime revert, I tested the most obvious structural
suspect. `MAX_N` is a *memory* ceiling, not a time ceiling: the one-GiB image law
binds near 92 672 vertices, so rows in `80 001..=92 672` are sparse enough to pass
`nnz <= 16n` yet large enough to carry a near-gigabyte mutable image. The hypothesis
was that this band is the expensive one.

It is not, and the measurement is decisive. The exact-window terminal region carries
no stage mark of its own, so its cost reads directly as `total − 22.win`. Running the
release probe with `SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1` on the five dev rows above
80 000 vertices:

| row | n | nnz | total s | terminal region s |
|---|---:|---:|---:|---:|
| `unitcommit_200_100_1_mod_8` | 146 830 | 476 332 | — | **0** |
| `cont6-qq` | 120 395 | 557 994 | 1.188 | **0** |
| `acopf_case9241pegase_qcqp` | 313 068 | 1 292 408 | 2.448 | **0** |
| `faclay75` | 272 878 | 1 379 706 | 2.548 | **0** |
| `gabriel10` | 244 056 | 1 148 210 | 3.378 | **0** |

Every one shows `22.win == final`, i.e. **zero** seconds in the terminal region:
`terminal_exchange` never fires on them. They are stopped by the anchor gate or the
`nnz` key long before the image law, so `MAX_N` is not a wall lever on them. Their
wall is `1.portfolio` — 1.60 s of `gabriel10`'s 3.38 s — a stage no change since
iteration 72 touches. The narrowing was measured, changed no ordering, and was
reverted rather than shipped on a false premise.

Also rejected, on prior evidence recorded in this repository's knowledge base: adding
an exact-window pass to a class row (the classic cap killer), and raising
`PRODUCTION_EXCHANGE_LEDGER` above 2 GiB.

## The cap trade

This submission trades nothing in score for its wall position. It removes a retained
large buffer and a skipped reset, so the worst case it can do is return to the
promoted tree's own allocation pattern on every row. Because `reset` charges the same
`2*n*w + 8n` either way, the deterministic run budget and the chosen permutation are
identical, which the 300-row record confirms.

If this tree still caps at the same ≈84 s position, the interpretation is crisp: the
lifetime divergence is exonerated, and the next controlled endpoint is the only
remaining lever inside the binding region — `PRODUCTION_EXCHANGE_LEDGER` 2 GiB → 1 GiB.
A 1 GiB-ledger tree has a completion receipt on record (`4bb9bdd2`, PR #715), and the
precharge is what bounds the terminal region's worst-case spend.

## Verification performed

- production candidate build via `scripts/local-candidate-build.sh`: clean.
- trusted parent build `cargo build --release -p matrices-fast --offline --locked`: clean.
- release ordering suite: **127 passed / 0 failed / 56 ignored**.
- complete 300-row development probe: `SCORE = 0.790254`, **300/300** `COUNTS` records.
- focused four-row reproduction of the sparse-terminal records: byte-identical.
- every changed path is under `src/ordering/`.

## Provenance

- Base: iter80 `84fd4c0`, submitted as `65c2e9d4-218f-4632-8089-2998e6e9ca1c` / PR #746.
- Promoted base used for the exact comparison: `05fa99b2cfc55cc9c7e166d4b348c454d30e6f3c`.
- This lane's completion receipt: `0e1edc92-c031-4db3-8470-b94cf8f142e7` / PR #722,
  14 min 02 s, score 0.840512.
- Knowledge base page: `src/ordering/memory/experiments/0289-restore-promoted-buffer-lifecycle.md`.
