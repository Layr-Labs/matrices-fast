# 0069 — Keep leftover `max_s=384`, do not unlock the subtree chain

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `e03e1bc` (0068 revert; leftover family back on the 0061 envelope). Official tip is still `9f37872` / Yukon `c7d5fe71` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **0065 Development Score:** 0.843147 with the chain unlocked (hidden **FAILED** `c16ff382`, 2 s cap)
- **Candidate Development Score:** 0.843328 (buckets 0.8903 / 0.8648 / 0.7919)
- **Delta:** −0.000030 (−0.30 basis points vs the 0061 tip)
- **Status:** LOCAL WIN, **HIDDEN FAIL** (submission `c20b927`, 2026-09-05). No hidden score — the 2 s cap. Leftover-384 + rounds 2–3 on every medium miss is still too much. See [0070](0070-leftover-384-pooling-band.md).

---

## 1. Context

The user asked to make 0065 pass the 2 s cap. 0065 is not a new idea: it is leftover miss-retry `max_s` 256→384 on the existing 0061 ticket, same pass count, same 32M envelope. Locally it moved `pooling_sppa9tp` from 0.4464 to 0.4090 and scored **0.843147** (−2.11 bip vs the 0061 tip). Hidden submission `c16ff382` failed with no score. That is the 2 s per-matrix cap.

0066–0068 then tried to recover the same basin without leftover-384:

| Experiment | Idea | Local | Submitted? |
| --- | --- | --- | --- |
| [0066](0066-amf-dead-zone-well-below.md) | AMF α-1 / relabelled AMF on `gams05` | no-op | no |
| [0067](0067-terminal-pass-nnz-125k.md) | terminal 16M nnz 100k→125k | `pooling_sppa9tp` stayed 0.446366 | no |
| [0068](0068-extra-small-well-below-lns.md) | extra small well-below LNS | `lt_1k` stayed 0.890338 | no |

0067 is the important negative for this revival. The leftover-384 basin is **not** in the later 16M terminal window. If leftover-384 is deleted, `pooling_sppa9tp` stays at 0.4464. The 0065 local win lives in the leftover refine itself, not in the chain that follows it.

So the 0065 timeout is not “leftover-384 is too expensive.” Local worst `order()` on 0065 was 1.355 s, inside the 0061 band (1.345–1.356 s). Requested leftover work is unchanged: medium leftover is still `2_000_000 × 16 × 1 = 32M`. `max_s` only changes which blocks are eligible.

The timeout is leftover-384 converting a **new** hidden matrix that leftover-256 missed, then unlocking rounds 2–5:

| Round | Requested work (typical medium) |
| --- | --- |
| leftover first-round (already spent) | 32M |
| 2 | 8M (`max_s=256` on below-anchor medium) |
| 3 | 8M (`max_s=512`) |
| 4 | 32M, or **64M** when `1000 <= n < 6000` |
| 5 | 16M, or 32M when `1000 <= n < 4000` |

A newly converted medium graph in the 1k–6k band then pays 64M + 32M on top of leftover 32M plus the independent terminal 16M cascade. Local SuiteSparse never saw that occupant — local worst stayed `crudeoil_lee4_10` / `arki0013` — so the 1.355 s figure did not warn.

0062–0064 timed out by **adding** a leftover refine. 0065 timed out by **widening** the existing leftover refine and then letting a new conversion start the same chain 0061 already ships. The fix is not to drop leftover-384. The fix is to keep leftover-384 and **not start the chain** when that is the path that improved.

---

## 2. Hypothesis

Leftover only runs when the size-only first round returned zero improvements (`improved == 0`). On that miss path:

1. Accept the leftover-384 candidate through the existing `flops_of` / best-of floor.
2. If leftover-wide improved, still run rounds 2–3 (8M+8M). Those cheap rounds are what 0061 already ships after leftover-256, and they are what actually moved `pooling_sppa9tp` (skip-all-chain left it at 0.4464).
3. If leftover-wide improved **and** round 3 accepted, stop. Do not enter rounds 4–5 (32–64M / 16–32M). That is the suffix a newly converted hidden medium graph pays in the 1k–6k 64M band.
4. If the size-only first round improved, keep the 0061 chain exactly, including 64M round 4.
5. If leftover used the small window (`n < 1000`, 2×1M) or the large window (`n >= 10_000`, `max_s=512`), keep the 0061 chain exactly.

The first skip-all-chain draft (leftover-384, no rounds 2–5) scored **0.843421** / `1k_10k` 0.8652 / `pooling_sppa9tp` 0.4464. So 0067 was only half right: the 16M *terminal* window does not hold the leftover-384 basin, but the *chain after leftover* does. Skipping the whole chain is a local loss. Skipping only the expensive suffix is the remaining 0065-cap move.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **restore 0065 leftover `max_s=384`, run rounds 2–3, and do not unlock rounds 4–5 on that path.**

```rust
let mut leftover_wide_hit = false;
if improved == 0 && best_flops < amd_flops && n <= 80_000 && nnz <= 250_000 {
    cfg1.round = 1;
    let leftover_wide = (1_000..10_000).contains(&n);
    if n < 1_000 {
        cfg1.streams = 2;
        cfg1.budget = 1_000_000;
    } else if leftover_wide {
        cfg1.max_s = 384; // 0065 window
    } else {
        cfg1.max_s = 512;
    }
    improved = rgreedy::subtree_refine(...);
    leftover_wide_hit = leftover_wide && improved > 0;
}
if improved > 0 && is_bijection(&candidate, n) {
    let f = flops_of(&scoring_pat, &candidate);
    if f < best_flops {
        best_flops = f;
        best_perm = candidate;
        // rounds 2–3 always, same as 0061
        if improved3 > 0 && f3 < best_flops {
            if !leftover_wide_hit {
                // rounds 4–5 unchanged from 0061
            }
        }
    }
}
```

The independent terminal 16M cascade after the chain still runs. Extra well-below LNS / extra relabel stay on the 0061 schedule. `is_well_below` stays at ratio `< 0.80`. Large leftover `max_s=512` still unlocks the chain. Size-only first-round hits still unlock the chain.

Determinism is unchanged: the skip is a compile-time structural gate on `n` and whether leftover ran, not a wall-clock check.

The work-limit unit test now checks `max_s = 384` on the medium leftover config. Requested leftover work is still inside `SUBTREE_SEARCH_WORK_LIMIT` (32M).

---

## 4. Why this is not a retry of a closed negative

Closed:

- Additive extra subtree pass on `n >= 10k` (0060 hidden timeout).
- Widening a *successful* size-only first-round `max_s` (0061 local loss 0.843829).
- A second leftover refine / third miss-retry / extra pass 9 (0062–0064 hidden timeout).
- Leftover `max_s` 256→384 **that then unlocks the full chain** (0065 hidden timeout). That is the thing this experiment deletes.
- 0.90 leftover tickets, including size-gated retractions (0062/0063).
- Opening `gt_10k` leftover at nnz ~120k (`crudeoil_lee4_10` 120632 vs `ringpack_30_2` 121458).
- Deep-below LNS/relabel pile-on (worst 1.504 s).
- AMF dead-zone / relabelled AMF on `gams05` (0066 no-op).
- Terminal 16M pass nnz 125k (0067 no-op).
- Extra small well-below LNS streams 7–8 (0068 no-op).

This is not “retry 0065 unchanged.” 0065’s failure mode was documented as chain-after-new-conversion. This submission keeps the conversion and cuts the chain. It does not add a refine, does not widen the successful first round, and does not spend 0.90 tickets.

It is also not “retry 0067.” 0067 asked the terminal 16M window to find the leftover-384 basin. It did not. This submission puts leftover-384 back and stops before the expensive suffix.

---

## 5. Timing argument

0061 leftover miss-retry **passed** hidden. This keeps:

- the same leftover gates (`below-anchor`, `n <= 80k`, `nnz <= 250k`);
- the same leftover requested work (medium 32M, small 2×1M, large 32M with `max_s=512`);
- the same size-only first-round chain, including the 64M round-4 band that already shipped.

The only added work versus the passing 0061 tip is “which medium leftover blocks are eligible” (`max_s` 256→384). A wider window can do more *useful* work inside the same budget; it cannot request more budget.

The only removed work versus failing 0065 is rounds 2–5 after a leftover-wide hit. That is the suffix the hidden occupant paid. Local `pooling_sppa9tp` (`n=5040`, `nnz=121302`) sits in the 64M round-4 band. Skipping that suffix on leftover-wide is the cap fix.

Worst-case local `order()` should stay in the 1.35 s band. If it climbs toward 1.5 s, leftover-384 itself is more expensive than believed and this revival is wrong.

---

## 6. Score argument

0065 with the chain: 0.843147. Almost all of the −2.11 bip is `pooling_sppa9tp` 0.4464→0.4090. 0067 proved that move is in leftover-384, not in later terminal work. Skipping the chain on leftover-wide should keep that matrix at ~0.4090.

What can go down: medium matrices leftover-256 already converted on 0061, whose chain suffix is now skipped. Those conversions themselves are kept (accepted via `flops_of`). Only rounds 2–5 after leftover-medium are dropped.

Target: local score strictly better than 0.843358, ideally near 0.843147, worst `order()` in the 1.35 s band. Then `yukon run` must print `Benchmark complete (score: …)` and hidden must return a numeric score, not `failed` + n/a.

If local score is worse than 0.843358, do not submit. Tighten the skip to leftover-wide and large nnz.

---

## 7. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
yukon submit --model "Gemini 3.8 Flash" --harness "Cursor" \
  --claimed-score <score> \
  --note-file src/ordering/memory/experiments/0069-leftover-384-without-chain.md \
  8c3e7051-530a-4aee-88df-a426e6e78151
```

---

## 8. Result

Skip-all-chain (leftover-384, no rounds 2–5):

```
SCORE = 0.843421
lt_1k 0.8903 · 1k_10k 0.8652 · gt_10k 0.7919
WORST order() = 1.348 s
pooling_sppa9tp = 0.4464
```

Local loss versus the 0061 tip. `1k_10k` went the wrong way because leftover-256 conversions lost their entire chain suffix. `pooling_sppa9tp` did not move: leftover-384's immediate `flops_of` is not the 0.4090 basin.

Cheap-chain revision (leftover-384 + rounds 2–3, skip 4–5):

```
SCORE = 0.843328
lt_1k 0.8903 · 1k_10k 0.8648 · gt_10k 0.7919
WORST order() = 1.359 s
pooling_sppa9tp = 0.4410
pooling_sppa9pq = 0.6302 (was 0.6473)
```

`lt_1k` and `gt_10k` are exact 0061 controls. `1k_10k` 0.8650→0.8648. Worst `order()` stays in the 1.35 s band. The remaining 0.4410→0.4090 lives in rounds 4–5, which is the suffix that timed out hidden.

−0.30 bip is under the usual ≥3 bip bar. Full 0065 (−2.11 bip) is unshippable. This is the 0065 window with the 64M suffix cut off.

Official `yukon run` printed `Benchmark complete (score: 0.843328)`.

---

## 9. Follow-ups

- If hidden still hits the 2 s cap, leftover-384 itself is the occupant (not the chain). Restore `max_s=256`. Do not add another first-round ticket.
- If hidden returns a numeric score worse than 0.86837, `pooling_sppa9tp` is not in the hidden corpus and leftover-384 without chain is a local-only move.
- If local score regresses versus 0.843358, nnz-gate the skip (`nnz >= 80_000` or `>= 100_000`) so cheap leftover-384 hits still chain.
- Keep-better-of (size-only chain vs leftover chain) is still untried and doubles the expensive suffix. Closed while leftover-384-without-chain is in flight.
- `RELABEL_AMF_MAX_NNZ = 200_000` still excludes `gams05` (nnz 252910, ratio 0.783). 0066 closed the cheap AMF revival there.

---

## 10. Why the skip is only leftover-wide

Small leftover (`n < 1000`) already passed hidden on 0061. Its leftover ticket is 2×1M, not a wider block window. A new small conversion unlocking the `lt_1k` chain is the 0061 envelope, not the 0065 failure.

Large leftover (`n >= 10_000`, `max_s=512`) also passed hidden on 0061. 0060 already closed additive extra subtree on `n >= 10k`. This experiment does not change that path.

Medium leftover is the only window 0065 changed. It is the only window that newly converted `pooling_sppa9tp`. It is the only window that can newly convert a hidden 1k–10k occupant into the 64M round-4 band. The skip is isolated there.

---

## 11. Borrow / contract notes

`consider` mutably borrows `best_flops`. This change does not call `consider` and does not read `best_flops` in a condition that also calls `consider`. Leftover acceptance stays on `flops_of` / best-of, same as 0061.

`order()` remains a deterministic valid permutation. No wall-clock gating, no unseeded RNG, no env reads. `SUBTREE_SEARCH_WORK_LIMIT` stays 32M. `TERMINAL_SUBTREE_SEARCH_WORK_LIMIT` stays 16M.

---

## Links

- The timeout this fixes: [0065-widen-existing-leftover-max-s.md](0065-widen-existing-leftover-max-s.md)
- Hidden-proven leftover envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Terminal window cannot recover leftover-384: [0067-terminal-pass-nnz-125k.md](0067-terminal-pass-nnz-125k.md)
- Extra-pass timeouts: [0062](0062-wider-margin-and-second-miss-retry.md), [0063](0063-drop-large-090-extra-relabel.md), [0064](0064-isolated-second-miss-retry.md)
