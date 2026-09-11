# 0157 — Close the last `best_flops` leak: stage 13's alternate-seed chains

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792573** (no change; 0 of 300 rows change their
  permutation or their exact flops)
- **Status:** invariant repair with zero dev effect; adopted

## Hypothesis

[0151](0151-best-flops-writeback-isolated.md) repaired three of the four
`best_flops` write-back leaks and shipped a `#[cfg(test)]` guard that still
fired on 4 dev rows. It identified the residue as **stage 13**, the
alternate-seed PEO chains: the stage assigns `best_perm = cur` while comparing
against a chain-local `leader_flops` and never touches `best_flops`.

Everything downstream of stage 13 — the terminal transplant, `15.minl`,
`15.peel`, the final refiners, the five-pass block and the completion block —
admits a candidate on `f < best_flops`. A `best_flops` above the truth admits a
donor whose *true* score is worse than the incumbent's, which is a strict
regression inside a pipeline whose entire safety argument is that regressions
are structurally impossible. Closing the leak should therefore be free or
positive on score and weakly work-reducing.

## What changed

One statement in `src/ordering/mod.rs`, at the end of the alternate-seed block:

```rust
best_flops = best_flops.min(leader_flops);
```

`leader_flops` is the **exact** score of `best_perm` at that point:

- it is initialised `let mut leader_flops = score(&best_perm);`
- the only assignment inside the loop is
  `if cur_flops < leader_flops { leader_flops = cur_flops; best_perm = cur; }`,
  and `cur_flops` is the round-final `fin`, which is `Σ c_j²` over `cur`'s own
  permuted column counts — the same quantity `score` computes;
- both early `break`s leave `cur` and `cur_flops` in step, and a chain that
  never completes a round keeps `cur_flops = u64::MAX` and cannot win.

So the write-back is exact, not an approximation, and it is applied with `min`
so it can only ever lower `best_flops`.

## Result

**Dev score is unchanged to the last digit** (0.792573, fill 0.924518), and an
exact per-row flops comparison against the base shows **0 of 300 rows moved**.
The `STALE_BEST_FLOPS` guard now fires on **0 of 300** rows, down from 4 — the
invariant is closed on this corpus.

The four rows the guard used to fire on:

| row | n | nnz | stale `best_flops` | true |
|---|---:|---:|---:|---:|
| `maxcsp-ehi-85-297-71` | 2372 | 207996 | 1112735756 | 1111065177 |
| `mpbp_15` | 9858 | 31692 | 1209494 | 1205050 |
| `kall_circlesrectangles_c6r39` | 1306 | 5378 | 74898 | 74892 |
| `syn40hfsg` | 1022 | 2624 | 8169 | 8155 |

All four pass stage 13's gate (`n >= 16 && n <= PEO_ALT_MAX_N &&
n + nnz < PEO_ALT_LEDGER && !peo_alt_danger`), which confirms the attribution:
every leak the guard saw came from this stage.

## Why it won / lost

Neither, on dev: the bad-admission window is real but unexercised here. On each
of the four rows the donor pools downstream happen to contain no candidate
scoring strictly between the true incumbent and the stale bound, so the wrong
acceptance test never actually fires. It is a correctness guarantee, not a dev
score change.

## Work envelope

A complete static audit of every `best_flops` read after the write-back — 40
sites in `mod.rs` between the alternate-seed block and the end of the pipeline —
finds only three shapes, and a **lower** `best_flops` is monotone-safe in all
three:

1. `if f < best_flops { … }` acceptance tests (the large majority) — strictly
   harder to satisfy, so weakly fewer admissions and no added work;
2. `best_flops = best_flops.min(cur_flops)` write-backs — idempotent under a
   lower incoming value;
3. `best_flops` passed as a **pruning bound** into `rgreedy::search` — a lower
   bound prunes strictly more.

The three delta-ratchets downstream of the stage (`best_flops_before_final`,
`before_five`, `before_comp`) each arm on a strict improvement measured from
the value at their own entry, so a lower entry value makes them **harder** to
arm, never easier. The three ratchets that unlock extra work on a strict
improvement upstream (`part_extra`, `part_extra2`, `credits` /
`core_path_improved`, and the `max_blocks` 2→4 doubling) all sit before stage
13 and cannot see this write.

**The caveat that no static audit covers:** on a row where the stale window
*was* open, refusing a donor changes the permutation handed to `15.minl` /
`15.peel` and the terminal blocks. Their cost is not a function of the
incumbent's score alone, so the cap has to be measured, not argued.

## Follow-ups

- The `#[cfg(test)]` guard costs one extra `flops_of` per row, so any timing
  read off a `cfg(test)` build of this tree is inflated and must not be compared
  against a tree without it. The guard is absent from the shipped binary.
- With this change every stage that improves `best_perm` writes its score back.
  Any *new* stage must do the same, and the guard is the check.

## Links

- [0151](0151-best-flops-writeback-isolated.md) — the first three write-backs
  and the guard
- [0152](0152-stage13-retirement-isolated.md) — retiring stage 13 instead is a
  dev loss
