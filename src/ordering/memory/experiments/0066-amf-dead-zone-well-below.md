# 0066 — Two relabelled AMF seeds just above 200k nnz, well-below only

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `f65e51a` then leftover `max_s` restored to 0061. Official tip `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; leftover `max_s` restored)
- **Status:** LOCAL NO-OP. Base AMF α-1 and two relabelled AMF seeds left `gams05` at 0.783380. Score stayed **0.843358**. Not submitted. See [0067](0067-terminal-pass-nnz-125k.md).

---

## 1. Context

Four leftover-search extensions on the promoted 0061 tip all failed the hidden 2 s cap:

| Experiment | What changed vs 0061 | Hidden |
| --- | --- | --- |
| 0062 | 0.90 tickets + second leftover refine | `395313f1` failed |
| 0063 | 0.90 LNS + second refine | `2d954e54` failed |
| 0064 | second leftover refine only | `71f76302` failed |
| 0065 | leftover `max_s` 256→384, **no extra refine** | `c16ff382` failed |

0065 is the important negative. Same pass count as the promoted tip. The only delta is a wider leftover window on first-round misses. That window can convert a new medium matrix and **unlock the rest of the subtree chain**. A hidden matrix that 0061 left as a miss then pays for rounds 2–5 plus terminal work and dies.

Leftover-search that unlocks a new chain is closed. This submission restores leftover `max_s = 256` and leaves the subtree family alone.

The AMF sweep in `order()` already has a documented dead zone: three α variants below `nnz = 130k`, one α-1 above `nnz = 400k`, **nothing in between**. That hole exists because `nuclear104` (`n=39098`, `nnz=257806`, ratio 1.000) sits there and already loads the candidate stack. Ties in that band must stay out.

`gams05` (`n=17364`, `nnz=252910`, ratio **0.783**, local `order()` ~1.24 s) sits in the same nnz band and is well-below the AMD anchor. It already gets base AMF α5 / α2 / default. It does not get the sweep α-1 pass, and relabelled AMF stops at `RELABEL_AMF_MAX_NNZ = 200k`.

---

## 2. Hypothesis

0056 / 0061: conversion tracks the AMD-anchor margin. Ties in the 130k–400k band are a timing bomb (`nuclear104`). Well-below incumbents in that band are the ones that still pay for a different AMF objective.

One AMF α-1 pass is not a subtree refine and does not unlock the leftover chain. It is a single `consider` on a best-of floor. Score risk is zero. Time risk is one min-fill sweep on well-below graphs only.

`gt_10k` weight is 0.40 over 45 matrices. One real conversion there is worth more than a pile of `lt_1k` leftovers.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **two relabelled AMF tickets just above `RELABEL_AMF_MAX_NNZ`, well-below only.**

A base AMF α-1 in the 130k–400k dead zone was a local no-op on `gams05` (already has α5/α2/default). 0004: the lever is a different numbering, not another α on the same graph.

```rust
if is_well_below(...) && n < 25_000
    && nnz > 200_000 && nnz <= 280_000
{
    // two relabel seeds, AMF α-1 then α5, compose back, best-of
}
```

`nuclear104` (`n=39098`) and `nuclear10a` (`nnz=163816`) stay out. `gams05` (`n=17364`, `nnz=252910`, ratio 0.783) is the local occupant.

Also restored leftover medium `max_s = 256` (0061). The 0065 window is closed.

Unchanged: relabelled-AMF ceiling 200k; three-pass sweep below 130k; one-pass sweep at `nnz >= 400k`; subtree leftover schedule.

Determinism is unchanged: the new gate is a pure function of `(n, nnz, best_flops, amd_flops)`.

---

## 4. Why this is not a retry of a closed negative

Closed:

- Additive extra subtree pass on `n >= 10k`.
- Widening a successful first-round `max_s`.
- A second leftover refine, or widening leftover `max_s` on the miss path.
- 0.90 leftover tickets.
- Opening `gt_10k` subtree leftover at nnz ~120k (`crudeoil` / `ringpack`).
- Deep-below subtree pile-on.
- Extra AMF on **ties** in the 130k–400k band (`nuclear104`).

This is one AMF pass, well-below only, in a band the comments already named. It does not add a `subtree_refine`.

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

---

## 6. Expected local result

If α-1 is a new minimum on `gams05`, `gt_10k` moves and the aggregate should drop. One `gt_10k` matrix is worth on the order of 0.002 if the ratio move is large (0004). A small move on `gams05` may be only 1–2 bip. If the pass does not beat the existing AMF α5/α2/default stack, the score stays 0.843358 and this does not ship.

Worst `order()` must stay in the 0061 band (1.35 s class). `gams05` starts ~1.24 s; one extra AMF pass has to leave it under the local worst case by a wide margin. `nuclear104` is a tie and must not enter the branch.

---

## 7. Timing argument

0061 extra relabel already spends 12 AMF+AMD seeds on well-below `n >= 10k` with `nnz <= 100k` and **promoted**. This pass is one AMF on a sparser-or-denser band (`130k–400k`) and only when ratio `< 0.80`. The local occupant of that band that is well-below is `gams05` at 1.24 s, not `crudeoil_lee4_10` (nnz 120632, outside this gate) and not `nuclear104` (tie).

---

## 8. Follow-ups

- If this times out, the 130k–400k AMF dead zone stays closed even on well-below graphs.
- If it is a no-op locally, do not submit; try more small-graph LNS or a different non-subtree lottery.
- Raising `RELABEL_AMF_MAX_NNZ` from 200k to 260k would add many AMF passes on `gams05` and is a larger timing bet than this single pass.
- Leftover `max_s` and extra leftover refines stay closed.

## Links

- Hidden-proven tip: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Leftover unlock timeouts: [0065](0065-widen-existing-leftover-max-s.md), [0064](0064-isolated-second-miss-retry.md)
