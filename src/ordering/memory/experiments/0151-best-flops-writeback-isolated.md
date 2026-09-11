# 0151 — Repair the stale-`best_flops` invariant, and nothing else

- **Date:** 2026-09-10
- **Score:** 0.792439 → **0.792439** (no change; 0 of 300 rows change their
  permutation or their exact flops)
- **Status:** invariant repair with zero dev effect; submitted as the second
  arm of a cap bisect

## Hypothesis

The terminal cross-candidate transplant admits a donor on `f < best_flops`, so
`best_flops` must be the exact score of the current `best_perm`. Several
earlier stages improve `best_perm` while tracking its score only in a
round-local variable, which leaves `best_flops` **too high** and opens a window
in which a donor *worse* than the incumbent is admitted.

## What changed

`src/ordering/mod.rs`, three write-backs and one test-only guard:

- stage 11's core-candidate loop already has the exact score in `f`; it now
  writes it back instead of only replacing `best_perm`;
- the two monotone `lt_1k` refiners return only a permutation, so the score is
  re-derived after them (`best_flops.min(score(&best_perm))`);
- both PEO chains keep their round-final exact score in `peo_true_flops` and
  write it back once the chain ends;
- a `#[cfg(test)]` guard immediately before the transplant prints
  `STALE_BEST_FLOPS` whenever `best_flops != score(&best_perm)`.

## Result

Dev score is **unchanged to the last digit**, and an exact per-row flops
comparison against the base shows **0 of 300 rows moved**. The window the
repair closes is not exercised by any dev row's donor set.

**The guard still fires on 4 rows after the repair**, all with `best_flops`
above the truth:

| n | nnz | `best_flops` | true | gap |
|---:|---:|---:|---:|---:|
| 2372 | 207996 | 1112735756 | 1111065177 | 1670579 |
| 9858 | 31692 | 1209494 | 1205050 | 4444 |
| 1022 | 2624 | 8169 | 8155 | 14 |
| 1306 | 5378 | 74898 | 74892 | 6 |

**The remaining leak is stage 13, the alternate-seed PEO chains.** That stage
assigns `best_perm = cur` while comparing against a chain-local `leader_flops`
and never touches `best_flops`. So the invariant is only fully restored by
either writing stage 13's score back or retiring the stage.

Note that the guard itself costs one extra `flops_of` per row, so **any timing
read off a `cfg(test)` build of this tree is inflated** and must not be
compared against a tree without it. The guard is absent from the shipped
binary.

## Why it is submitted alone

Two of my trees died on the 2 s per-matrix cap while the base they descend from
passes. The diff contained three changes; the first — removing the narrow
stage-1b force-adoption windows — has now been submitted alone and **cleared
the cap**. This is the second of the two survivors, so this submission decides
the bisect in either direction: if it clears the cap, the culprit is the
stage-13 retirement; if it does not, the culprit is this repair, which would be
a genuinely surprising result about a change that touches no dev row at all.

## Follow-ups

- Close the stage-13 leak, then re-measure — the 4 leaking rows are the only
  dev rows where the transplant's admission test is currently wrong.

## Links

- [0150](0150-stage1b-window-removal-isolated.md) — the first arm of the bisect.
