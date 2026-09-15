# 0289 — restore the promoted `Game` buffer lifecycle

- **Date:** 2026-09-15 (iter81)
- **Base:** iter80 `84fd4c0` / submitted as `65c2e9d` / PR #746.
- **Change:** drop the sparse-image retention and the reset-first dead-copy
  skip; the working image is fully initialized at construction and released
  above the promoted 160 MB pool ceiling.
- **Public score:** unchanged from iter80, **0.790253862267** (300/300 rows).
- **Status:** verified, ready for submission.

## What the iter80 submission actually did

The handoff prompt did not record it, but iter80 was submitted as `65c2e9d` /
PR #746 at 15:11:58Z and cap-failed like its three predecessors. Workflow
`34986819819` entered Benchmark at `15:15:36.8320742Z` and reported

```
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

at `15:17:00.9380008Z`, i.e. **84.1 s**. The four same-day endpoints are now:

| tree | submission | Benchmark time to cap |
|---|---|---:|
| iter77 | `b84237f` | 84.509 s |
| iter78 | `782a26d` | 84.681 s |
| iter79 | `26f00f8` | 84.360 s |
| iter80 | `65c2e9d` | 84.106 s |

All four carry the same error string: a genuine **2.0 s wall kill on a hidden
matrix**, not a memory-cap failure. Nothing in this sequence orders by tree cost.

## A refuted hypothesis, recorded so it is not retried

A promising-looking lead was that `rgreedy::MAX_N` had moved from `80_000` to
`1 << 17` after the iteration-72 tree (`a541700`, submission `0e1edc92`, PR
#722) that is this lane's only **completion** (14 min 02 s, 0.840512, rejected
by 1e-6). The argument was that the one-GiB image law is a *memory* law, not a
*time* law, so the rows in `80_001..=92_672` — sparse enough for `nnz <= 16n`,
large enough for a near-gigabyte image — would be the expensive ones.

That argument is **wrong**, and the measurement that kills it is cheap. Only the
`1.portfolio` / `9.reduce` / `1b.indep` phase marks exist before the terminal
region, and the region itself is unmarked, so the terminal work can be read
directly as `total - 22.win`. Running the release probe on the five dev rows
above 80 000 vertices with `SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1`:

| row | n | nnz | total s | terminal region s |
|---|---:|---:|---:|---:|
| `unitcommit_200_100_1_mod_8` | 146 830 | 476 332 | — | 0 |
| `cont6-qq` | 120 395 | 557 994 | 1.188 | 0 |
| `acopf_case9241pegase_qcqp` | 313 068 | 1 292 408 | 2.448 | 0 |
| `faclay75` | 272 878 | 1 379 706 | 2.548 | 0 |
| `gabriel10` | 244 056 | 1 148 210 | 3.378 | 0 |

**Zero** terminal seconds on every one of them: `22.win` and `final` are
identical, so `terminal_exchange` never fires. These rows fail the anchor gate
or the `nnz` key long before the image law. `MAX_N` is therefore not a wall
lever on them, and the one-line narrowing back to `80_000` was reverted. The
`1.portfolio` stage — which no change since iteration 72 touches — carries
1.60 s of `gabriel10`'s 3.38 s.

The same probe output also shows why a *local* cap story has to be read
relative to day and host: these rows exceed 2.0 s locally on the unmodified
tree, so the enforced cap is a graded-host budget, not a local one.

## The convergence argument that survives

The promoted base `05fa99b` is the last tree that scored 0.840511 on a live
corpus, and both it and iter72 (`0e1edc92`) **completed** while every submission
since has not. Comparing the promoted ordering sources (fetched from
`repos/.../contents/src/ordering/...?ref=05fa99b2cfc...`) against iter80 gives a
much smaller difference than the handoff implies:

- `MAX_N` is `1 << 17` in **both**; the promoted tree is not the `80_000` tree.
- The sparse `Pristine` representation, `SPARSE_PRISTINE_MIN_WORDS`,
  `new_sparse`, the `XCH_ALLOC` policy and `PRODUCTION_EXCHANGE_LEDGER` are
  present in both.
- `mod.rs` has 37 non-comment changed lines against the base and every one of
  them is a `#[cfg(test)]` seam default, except `peo_alt_seeds()`.
- `rgreedy/window_dp.rs` differs in **15 hunks, all test-only instrumentation**
  plus `PRODUCTION_XCH_ALLOC: 0 -> 1`.

So the only functional production difference between the promoted tree and
iter80 was the four buffered-image changes below, all in `rgreedy.rs`.

## Change

1. `use_sparse_pristine` no longer keeps a `1 << 27`-word (1 GiB) mutable image
   alive per thread between sequential window calls; ordinary and sparse games
   share one 160 MB pool ceiling via `ADJ_POOL_MAX_WORDS`.
2. `Game::new_sparse` materializes its image into a **fresh**
   `alloc_parallel_zeroed_u64_vec` (the buffer it took from the pool is
   released), so its first `reset` can never read arbitrary recycled words.
3. `Pristine::game_reset_first` / `Game::new_reset_first_with_degrees` /
   `assemble(…, initialize_adj)` are gone. `Pristine::game()` calls
   `Game::new_with_degrees`, which fills the image with `copy_from_slice`
   before any reader can see it.
4. The `sparse_adj_is_pristine` flag and its first-`reset` fast path are
   removed; every `reset` on the sparse path clears and rebuilds, exactly as in
   the promoted tree.

`rgreedy.rs` net **−55 lines**, `window_dp.rs` one call site. No search budget,
admission gate, charge model, or acceptance rule changes, and `reset` still
charges the identical `2·n·w + 8n` either way. The `SSI_XCH_EAGER_ADJ` seam is
retired with the constructor it priced; the old behaviour remains reproducible
from commits `0503544` and `0696f9a`.

## Result

The four sparse-terminal movers reproduce their exact flop records, and their
phase profile is unchanged:

| row | candidate flops (before → after) | total s (before → after) |
|---|---:|---:|
| `nuclear104` | 78 332 024 → 78 332 024 | — |
| `transswitch2736spr` | 7 281 507 → 7 281 507 | 1.779 → 1.858 |
| `transswitch2383wpr` | 3 861 509 → 3 861 509 | 2.506 → 2.065 |
| `gams05` | 3 259 322 396 → 3 259 322 396 | 3.400 → 3.269 |

The complete 300-row `SSI_MARK_NOSCORE=1` probe scores **SCORE = 0.790254**
with **300 `COUNTS` records**, the iter80 value. The wall column is reported
only to show it does not order the arms consistently on a shared host; it is
**not** claimed as a speedup.

## Why this is the right narrow next step

This is the convergence form of the same causal test iter80 attempted. Iter80
removed the retired fork's source scaffolding; this removes the last production
divergence from the tree that last completed. If the cap kill was caused by a
retained large image or by reading a buffer whose initialization contract
changed, this fixes it directly. If the kill is instead a property of the daily
corpus and the 2 GiB ledger's worst-case spend, this will cap-fail at the same
84-second position, and the next controlled endpoint is the ledger itself
(`PRODUCTION_EXCHANGE_LEDGER` 2 GiB → 1 GiB), which has a completion receipt on
record (`4bb9bdd2`, PR #715).

## Verification

- production candidate build: clean.
- release suite: **127 passed / 0 failed / 56 ignored**.
- full 300-row dev probe: `SCORE = 0.790254`, 300/300 `COUNTS` records.
- `git diff --stat`: `src/ordering/rgreedy.rs` −55 net, `window_dp.rs` one
  line; every changed path is under `src/ordering/`.

## Evidence

- iter80 submission: `65c2e9d4-218f-4632-8089-2998e6e9ca1c`, PR #746, workflow
  `34986819819` (cap kill at 84.1 s).
- promoted base fetched for the comparison:
  `05fa99b2cfc55cc9c7e166d4b348c454d30e6f3c`.
- completion receipt: `0e1edc92-c031-4db3-8470-b94cf8f142e7`, PR #722, 14 min
  02 s, 0.840512.
- local probe arms: `.session-backup/probe-iter81`,
  `.session-backup/probe-iter81B`, `.session-backup/iter81-B-full.log`.

## Links

- [0288 remove the retired fork's global suffix scaffolding](0288-restore-direct-suffix-control-flow.md)
- [0283 sparse first-reset and buffer retention](0283-sparse-first-reset-and-buffer-retention.md)
- [0280 reset-first adjacency copy](0280-reset-first-adjacency-copy.md)
