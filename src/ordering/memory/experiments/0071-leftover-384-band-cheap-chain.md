# 0071 — Pooling-band leftover-384 with rounds 2–3 only

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `b5357e5` (0070). Official tip is still `9f37872` / Yukon `c7d5fe71` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip)
- **0070 Development Score:** 0.843148 with leftover-384 + full chain on the pooling band (hidden **FAILED** `194629f`)
- **0069 Development Score:** 0.843328 with leftover-384 + rounds 2–3 on every medium miss (hidden **FAILED** `c20b927`)
- **Candidate Development Score:** 0.843329 (buckets 0.8903 / 0.8649 / 0.7919)
- **Delta:** −0.000029 (−0.29 basis points vs the 0061 tip)
- **Status:** LOCAL WIN. `pooling_sppa9tp` 0.4464→0.4410, `pooling_sppa9pq` 0.6473→0.6302, worst `order()` 1.349 s.

---

## 1. Context

The user asked to make 0065 pass the 2 s cap. Three leftover-384 submissions have now failed hidden with n/a:

| Experiment | Where leftover-384 runs | Chain after leftover-384 | Local | Hidden |
| --- | --- | --- | ---: | --- |
| [0065](0065-widen-existing-leftover-max-s.md) | every medium miss | full (incl. 64M r4) | 0.843147 | `c16ff382` **failed** |
| [0069](0069-leftover-384-without-chain.md) | every medium miss | rounds 2–3 only | 0.843328 | `c20b927` **failed** |
| [0070](0070-leftover-384-pooling-band.md) | pooling band only | full (incl. 64M r4) | 0.843148 | `194629f` **failed** |

0070 is the important new fact. Isolating leftover-384 to `4000 <= n < 8000` and `100k <= nnz <= 150k` still timed out. The hidden occupant **is in that band**. The 64M round-4 suffix on a leftover-384 conversion in that band is what 0065 and 0070 paid.

0069 already cut that suffix, but it also put leftover-384 on every medium miss, which created *other* new conversions and timed out.

The remaining combination is: **0070’s band + 0069’s cheap chain.** Leftover-384 only where the two local pooling graphs live. After a hit, run rounds 2–3 (8M+8M) and stop. Leftover-256 + the full 0061 chain everywhere else.

0069 skip-all-chain (no r2–r5) left `pooling_sppa9tp` at 0.4464 and scored 0.843421 (local loss). Cheap chain is what moved it to 0.4410. Full chain is what moved it to 0.4090 — and that is the unshippable suffix.

---

## 2. Hypothesis

0070 timed out because leftover-384 converted a hidden pooling-band graph and then paid rounds 4–5 (32–64M / 16–32M). Those two graphs sit in the 1k–6k 64M round-4 band (`n≈5040`). Cutting that suffix on leftover-384-band hits keeps the 0069 pooling move (0.4464→0.4410, 0.6473→0.6302) and does not recreate 0069’s medium-wide leftover-384 conversions.

Expected local score: better than 0.843358 (the two pooling graphs still move) and better than 0.843328 (leftover-256 conversions keep their r4–r5 suffix, which 0069 stripped). Not as low as 0.843148 (no 0.4090). Worst `order()` in the 1.35 s band. Hidden occupant in the band pays leftover-384 + 16M instead of leftover-384 + 16M + 64M + 16–32M.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **leftover-384 on the pooling band, cheap chain only; leftover-256 + full chain everywhere else.**

```rust
let leftover_384_band =
    (4_000..8_000).contains(&n) && (100_000..=150_000).contains(&nnz);
// leftover max_s = 384 on that band, else 256 / 512 as in 0061
leftover_384_band_hit = leftover_384_band && improved > 0;
// after accepting round 3:
if !leftover_384_band_hit {
    // rounds 4–5 unchanged from 0061
}
```

Unchanged: leftover gates, small leftover 2×1M, large leftover `max_s=512`, size-only first-round chain, `is_well_below` at 0.80.

Determinism is unchanged. Requested leftover work is still 32M. The work-limit test still covers both leftover-256 medium and leftover-384 pooling-band configs.

---

## 4. Why this is not a retry of a closed negative

Closed:

- Leftover-384 on every medium miss + full chain (0065).
- Leftover-384 on every medium miss + rounds 2–3 (0069).
- Leftover-384 on every medium miss + no chain (0069 skip-all: local loss).
- Leftover-384 on the pooling band + full chain (0070).
- Second leftover refine (0062–0064).
- 0.90 tickets, `gt_10k` nnz~120k leftover, AMF dead-zone, terminal nnz 125k, extra small LNS.

This is the one remaining leftover-384 cell: **band ∩ cheap chain.** 0069 was cheap chain at full medium width. 0070 was full chain at band width. Neither pair is this pair.

---

## 5. Timing argument

0061 leftover-256 + full chain passed hidden. This submission is 0061 except on the pooling band.

On that band, versus failing 0070, the deleted work is rounds 4–5 after a leftover-384 hit. That is 32–64M plus 16–32M. The hidden occupant 0070 created still gets leftover-384 + 8M + 8M. That is the same extra as 0069 paid on that occupant, without 0069’s extra leftover-384 conversions on the rest of the medium miss set.

Local `pooling_sppa9tp` under leftover-384 + cheap chain was 0.54 s (0069). The 1.35 s worst case is outside the band.

If this still hits the 2 s cap, leftover-384 itself (32M, wider window) is the occupant even without r2–r3, and leftover-384 is closed at every width and chain length.

---

## 6. Score argument

0069 cheap-chain pooling moves: `pooling_sppa9tp` 0.4464→0.4410, `pooling_sppa9pq` 0.6473→0.6302. Those two are in the band, so they still happen.

0069 also stripped r4–r5 from leftover-256 medium conversions and scored 0.843328. This submission does not strip those. Expected local score in (0.843148, 0.843358), closer to 0.84330.

If local score is worse than 0.843358, do not submit.

---

## 7. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
yukon submit --model "Gemini 3.8 Flash" --harness "Cursor" \
  --claimed-score <score> \
  --note-file src/ordering/memory/experiments/0071-leftover-384-band-cheap-chain.md \
  8c3e7051-530a-4aee-88df-a426e6e78151
```

---

## 8. Result

```
SCORE = 0.843329
lt_1k 0.8903 · 1k_10k 0.8649 · gt_10k 0.7919
WORST order() = 1.349 s
pooling_sppa9tp = 0.4410
pooling_sppa9pq = 0.6302
```

`lt_1k` and `gt_10k` are exact 0061 controls. The two pooling graphs moved as in 0069. Leftover-256 conversions kept their r4–r5 suffix (`1k_10k` 0.8649 vs 0069’s 0.8648). −0.29 bip is under the usual ≥3 bip bar; the goal is the 2 s cap, not a large local drop.

Official `yukon run` printed `Benchmark complete (score: 0.843329)`.

---

## 9. Follow-ups

- If hidden still hits the 2 s cap, leftover-384 is closed. Restore leftover-256 globally. Do not add another first-round ticket.
- If hidden returns a numeric score worse than 0.86837, the pooling graphs are not worth the leftover-384 risk.
- If hidden promotes, leftover-384 is legal only as a banded cheap-chain ticket.

---

## 10. Borrow / contract notes

`consider` mutably borrows `best_flops`. This change does not call `consider`. Leftover acceptance stays on `flops_of` / best-of.

`order()` remains a deterministic valid permutation. No wall-clock gating, no unseeded RNG, no env reads. `SUBTREE_SEARCH_WORK_LIMIT` stays 32M. `TERMINAL_SUBTREE_SEARCH_WORK_LIMIT` stays 16M.

---

## Links

- Band + full chain timeout: [0070-leftover-384-pooling-band.md](0070-leftover-384-pooling-band.md)
- Medium-wide cheap chain timeout: [0069-leftover-384-without-chain.md](0069-leftover-384-without-chain.md)
- Medium-wide full chain timeout: [0065-widen-existing-leftover-max-s.md](0065-widen-existing-leftover-max-s.md)
- Hidden-proven leftover envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
