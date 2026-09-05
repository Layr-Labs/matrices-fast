# 0067 — Open the extra terminal 16M pass to nnz 125k on n < 10k

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** leftover family restored to 0061 (`max_s = 256`). Official tip `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **Status:** LOCAL NO-OP. `pooling_sppa9tp` stayed 0.446366; `1k_10k` 0.864950. The 16M window does not find the leftover-384 basin. Not submitted. See [0068](0068-extra-small-well-below-lns.md).

---

## 1. Context

Leftover first-round search that converts a *new* matrix and then unlocks rounds 2–5 is closed. 0065 proved it: same pass count as the promoted tip, only `max_s` 256→384 on the miss path, hidden 2 s cap (`c16ff382`). 0062–0064 failed by adding a leftover refine. 0066 (AMF in the 130k–400k hole, then two relabelled AMF seeds on `gams05`) was a local no-op.

0061 already ships one extra terminal subtree ticket after the deep chain:

```
best_flops < amd && n < 10_000 && nnz <= 100_000 && n >= SUBTREE_MIN_N
4 blocks × 4M, max_s = 512, round = 8
```

Requested work 16M. It updates `best_perm` if flops drop. It does **not** start the leftover first-round chain.

`pooling_sppa9tp` (`n=5040`, `nnz=121302`, ratio 0.4464) and `pooling_sppa9pq` (`n=5030`, `nnz=120730`, ratio 0.630) sit just above 100k. 0064/0065 moved `pooling_sppa9tp` to 0.4090 via leftover `max_s = 384`, then died hidden because that leftover unlocks the chain. This ticket never unlocks that chain.

---

## 2. Hypothesis

A bounded terminal refine on two medium pooling graphs is not a leftover first-round. The 0065 failure mode was “new leftover conversion → full chain.” This pass runs after the chain, on the incumbent the chain already produced (or failed to touch), and stops.

`n < 10k` keeps `crudeoil_lee4_10` / `arki0013` / `ringpack_30_2` out. `nnz <= 125k` takes the two pooling graphs and leaves dense mediums (`meanvar` 171k, `maxcsp-ehi` 208k) out.

If the 16M window finds the same `pooling_sppa9tp` basin leftover 384 found, local score should land near 0.843148 (−2.10 bip) with 0061 timing.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **raise the extra terminal pass nnz cap 100k → 125k. Still `n < 10k`, below-anchor.**

```rust
if best_flops < amd_flops && n < 10_000 && nnz <= 125_000 && n >= SUBTREE_MIN_N {
    // existing 4 × 4M, max_s = 512, round = 8
}
```

Leftover first-round stays at 0061 (`max_s = 256`, one refine). 0066 AMF experiments removed (no-op).

Determinism unchanged. Requested work of this pass stays 16M.

---

## 4. Why this is not a retry of a closed negative

Closed:

- Additive extra subtree on `n >= 10k`.
- Widening a successful first-round `max_s`.
- A second leftover refine, or leftover `max_s` 256→384 (unlocks the chain).
- 0.90 leftover tickets.
- Opening `gt_10k` leftover at nnz ~120k.
- AMF dead-zone / relabelled AMF on `gams05` (local no-op).

This widens an existing **terminal** ticket’s nnz gate by 25k on `n < 10k` only. No new refine type. No leftover unlock.

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

---

## 6. Expected local result

Target: `pooling_sppa9tp` 0.4464→0.4090 and maybe `pooling_sppa9pq`. Aggregate near **0.843148** (−2.10 bip) if the 16M window hits. Worst `order()` should stay in the 1.35 s 0061 band: the new occupants are medium pooling graphs, not `crudeoil_lee4_10`.

If the pass finds nothing, score stays 0.843358 and this does not ship.

−2.10 is under the usual ≥3 bip bar. After four leftover hidden timeouts, a terminal ticket that can take the one known leftover win without unlocking the chain is the remaining leftover-adjacent move.

---

## 7. Timing argument

0061 already runs this 16M pass on every below-anchor `n < 10k` graph with `nnz <= 100k` and **promoted**. Adding two graphs at nnz ~121k is two more 16M refines. Local pooling graphs are not the worst case. Hidden risk is “a medium ~121k nnz graph already near 2 s.” That is smaller than “every leftover miss now runs a wider window and may start the chain.”

---

## 8. Follow-ups

- If this times out, do not widen this pass further and do not put it on `n >= 10k`.
- If it is a no-op, leftover `pooling_sppa9tp` is only reachable by unlocking the chain, which is closed.
- Extra small-graph LNS streams are still open and do not touch subtree.

## Links

- Extra pass origin: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Leftover unlock timeouts: [0065](0065-widen-existing-leftover-max-s.md)
- AMF no-op: [0066](0066-amf-dead-zone-well-below.md)
