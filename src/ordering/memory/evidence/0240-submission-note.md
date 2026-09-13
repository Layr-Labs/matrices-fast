# 0240 — the class block's `nnz` admission key: 200 000 → 300 000

## What changed

`src/ordering/mod.rs`, terminal class block. The gate that admits a row to the
exact window exchange (`rgreedy::subset_window_descent_step`, shipped shape
12/5/5, step 5, 4 GiB ledger) read

```rust
let terminal_exchange = n >= 6 && n <= class_n && nnz <= 200_000
    && nnz <= n.saturating_mul(16) && max_deg <= n / 2;
```

and the follow-up block (PEO extraction, sparse spans, and the dense/hub copy of
the same window exchange) repeated the literal `nnz <= 200_000`. This submission
raises **one** of those two copies, the class block's, to `300_000`. The
follow-up copy is deliberately left at `200_000` (measured inert, see below).
Both values are now named (`class_nnz` / `follow_nnz`) with a test-only seam
(`SSI_CLASS_NNZ` / `SSI_FOLLOW_NNZ`) whose defaults *are* the shipping values,
so the probe frame and the graded frame agree by construction. Nothing else in
`order()` changes; no other file is touched.

This is a structural admission key (dimension `n`, structural nonzeros `nnz`,
maximum column degree), never matrix identity, no lookup table, no corpus
fingerprinting: the new rows are admitted by their own `(n, nnz)` state.

## Why this key, and why it was the last unpriced one

The lane's largest measured win to date was moving the *same gate's* `n` ceiling
(`rgreedy::MAX_N` 12 000 → 25 000), worth −9.4e-4 dev by admitting a band of
rows to exactly this exchange. The `nnz` clause of that gate had never been
priced. A census of the public dev corpus (`corpus/dev/patterns.jsonl`, all 300
rows, off-diagonal nonzeros, column degrees after dropping the diagonal):

* 14 of the 300 dev rows carry more than 200 000 off-diagonal nonzeros;
  13 of those 14 are in the `gt_10k` bucket, i.e. the 40 %-weight bucket;
* eight of the fourteen have `n > class_n` (25 000), so the `n` clause hides
  them no matter what this key says (`faclay75`, `acopf_case9241pegase_qcqp`,
  `gabriel10`, `unitcommit_200_100_1_mod_8`, `cont6-qq`, `transswitch2736spr`,
  `transswitch2383wpr`, `nuclear104`);
* of the six that reach this key, five are dense/hub (`nnz > 16n ||
  max_deg > n/2`: `pooling_sppc1pq`, `pooling_sppb5pq`, `pooling_sppc3pq`,
  `kissing2`, `maxcsp-ehi-85-297-71`) and therefore hit the *follow-up* block's
  own copy of the key, not this one;
* **exactly one dev row fails this key alone: `gams05`** (n = 17 364,
  nnz = 252 910, `nnz/n` = 14.6, maxdeg 68). It received no class exchange at
  all before this change.

## Measurements (one binary, one session, 4 vCPU, graded-closest frame)

All arms below were run back-to-back on the same freshly built probe binary,
`taskset -c 0-3`, with `SSI_MARK_NOSCORE=1` — the frame whose per-row wall clock
matches the graded build, since the probe's phase marks are `#[cfg(test)]` and
each pays one extra full scoring pass that the shipped worker never pays. 300
dev rows each; `lt_1k` / `1k_10k` weights are 0.30/0.30 and `gt_10k` 0.40.

| arm | key(s) | SCORE | worst `order()` | `gams05` |
|---|---|---|---|---|
| control (shipped tree) | 200 000 / 200 000 | **0.790322** | 1.483 s | 0.895 s, ratio 0.5274 |
| A: class key lifted | 10 000 000 / 200 000 | **0.790308** | 1.479 s | 1.312 s, ratio **0.5262** |
| B: both keys lifted | 10 000 000 / 10 000 000 | **0.790308** | 1.480 s | 1.321 s, ratio 0.5262 |

* **Arm A is the device: −1.4e-5 dev score**, entirely from one row.
  `gams05` improves **0.23 % relative** (0.5274 → 0.5262) for **+0.41 s** of
  its own time; the bucket geomean `gt_10k` moves 0.6824 → 0.6823 and the
  corpus worst row is unchanged within noise (the exchange is capped by its own
  ledger, so it cannot run away).
* **Arm B (the follow-up copy of the key) is exactly zero**: all five dense rows
  above the key read the *same ratios to four decimals* as control
  (`pooling_sppc3pq` 0.2822, `pooling_sppb5pq` 0.4077, `pooling_sppc1pq` 0.1683,
  `kissing2` 1.0000, `maxcsp-ehi-85-297-71` 0.8040) and the same times within
  ±0.03 s. Those rows are not bound by `nnz`; the follow-up block is gated
  inside by `nnz_l <= followup_factor`. The copy therefore stays at 200 000 —
  no reason to buy a key that changes nothing.
* The on-dev gap between 300 000 and 10 000 000 is empty: the next rows above
  `gams05` are `nuclear104` (257 806, hidden by `n > 25 000`) and
  `transswitch2383wpr` (277 562, n = 59 853, likewise), then
  `transswitch2736spr` (331 010) is already above 300 000. So the shipping value
  reproduces arm A's measured number on dev exactly.

## Official local sandboxed harness (the preflight)

`bash scripts/local-candidate-build.sh && cargo run --release` — the repo's own
benchmark command, candidate worker built and run inside the bubblewrap
boundary, 2 s per-matrix cap enforced by the trusted parent:

* **status OK, 300/300 rows**, score **0.790296**, tiebreak fill **0.923238**;
  buckets `0.8873 / 0.8373 / 0.6823`; `results.tsv:1789289499`.
* Against the same tree's own official run of the previous iteration
  (`0.790309 / 0.923267`, `results.tsv:1789288143`) that is **−1.3e-5 in the
  graded frame**, consistent with the in-frame −1.4e-5.

## Cost and risk (stated in full)

Every adoption inside the class block is a *strict* flop decrease against the
incumbent, so this change cannot make any row's ratio worse on any corpus. The
entire price is per-row seconds on rows the key newly admits: on dev that is one
row, costing +0.41 s on a row that reads 1.31 s against a 2.0 s cap local worst
of 1.479 s. The work is bounded by the block's own 4 GiB ledger, which charges
the window search before it runs, so a pathological admitted row cannot spend
unbounded time: it spends at most the ledger and returns the best strict gain it
reached. On the hidden corpus the newly admitted class is "n ≤ 25 000,
200 000 < nnz ≤ 300 000, not dense (`nnz ≤ 16n`), not hub (`maxdeg ≤ n/2`)" —
sparse, large-dimension, factor-heavy rows; the dev row of that class has 1.1 s
of measured headroom and uses 0.41 s of it.

## Reproduce

```
cd <repo>
CARGO_BUILD_JOBS=2 bash -lc 'bash scripts/local-candidate-build.sh && cargo run --release'
# in-frame A/B on one binary, one session:
taskset -c 0-3 env SSI_MARK_NOSCORE=1 bash target/probe-sandbox.sh run                    # control
taskset -c 0-3 env SSI_MARK_NOSCORE=1 SSI_CLASS_NNZ=10000000 bash target/probe-sandbox.sh run  # arm A
taskset -c 0-3 env SSI_MARK_NOSCORE=1 SSI_CLASS_NNZ=10000000 SSI_FOLLOW_NNZ=10000000 \
    bash target/probe-sandbox.sh run                                                     # arm B
```

Evidence (all in `src/ordering/memory/evidence/`): `0240-classnnz-control-4cpu.log`,
`0240-classnnz-A-4cpu.log`, `0240-classnnz-B-4cpu.log`,
`0240-official-run-classnnz300k.log`; the ledger entry is in
`src/ordering/memory/log.md` (iter58). The corpus census above is computed from
`corpus/dev/patterns.jsonl` (the corpus is not modified).
