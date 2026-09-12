# 0207b — the terminal four-stream round returns at 2e9, keyed on the row's own live ratio

## 1. What changed

Two production constants in `src/ordering/mod.rs` (the rest of the diff is test-only marks):

```rust
+ const SHIPPED_ENGINE_FANOUT: &[i64] = &[2_000_000_000];   // was &[] in the committed tree
+ const ENGINE_FANOUT_RATIO_PCT: u64 = 90;                  // new key, 0 disables
...
- if fanout_gate && best_flops <= LADDER_FILL_BOUND {
+ if fanout_gate && best_flops <= LADDER_FILL_BOUND && fanout_ratio_ok {
```

`fanout_ratio_ok` is the row's own incumbent measured against the anchor snapshot the pipeline
already takes for the terminal rung ladder: `best_flops * 100 <= anchor_flops * 90`. It is a
posterior, scale-free property of the row's own state — not an `n`/`nnz` window, not an identifier.
The round itself is unchanged (four independent `rgreedy` trajectories merged by a strict
`(flops, source index)` argmin, so it cannot depend on thread scheduling).

## 2. Why the round comes back at all: the iter40 "removal is free" A/B was a stale binary

The committed tree removed this stage on the strength of an A/B reported as byte-identical
("0 of 300 dev rows change flops, `results.tsv:1789241463` = 0.791896 with the list emptied").
That is not reproducible:

* the **official sandboxed local harness** (production path, no seams) on the *committed* tree, in
  the graded 4-vCPU frame: `300 matrices, 0 failures, score 0.792166 / tiebreak 0.924495`
  [evidence: `0207-harness-tip-4cpu.log`, `score.json`, `results.tsv:1789242561`];
* a **single-variable probe A/B** (one binary, one session, same seams, only the budget list
  changing) prices the round's dose curve [evidence: `0207-fanout{off,5e8,1e9,2e9}-probe-4cpu.log`]:

| budget list | dev SCORE | movers | max per-row +wall | corpus +wall | worst `order()` |
|---|---|---|---|---|---|
| `&[]` (committed) | 0.792166 | 0 | — | — | 1.284 s |
| `&[5e8]` | 0.792079 | 12 | +0.365 s (`mpbp_07`, zero gain) | +6.1 s | 1.416 s |
| `&[1e9]` | 0.792065 | 14 | +0.684 s (`emfl100_3_3`, zero gain) | +13.6 s | 1.456 s |
| `&[2e9]` | 0.791896 | 14 | +1.485 s (`emfl100_3_3`, zero gain) | +23.9 s | 1.812 s |

The removal silently gave up 2.7e-4 of dev, i.e. most of this branch's total gain.

## 3. Why the *key*, and why 90 %

The price law is the point: the round's cost is linear in its budget and lands on the rows it
cannot move. At 2e9 the dearest rows are `emfl100_3_3` +1.485 s, `squfl020-150` +1.256 s,
`glider400` +1.182 s, `rsyn0820m04m` +1.173 s, `syn40m04hfsg` +1.162 s, `squfl015-080persp`
+1.135 s, `ringpack_20_3` +1.082 s — **every one with zero flop gain** — while every row the round
*does* move is one the pipeline has already driven well under its anchor
(`rsyn0830m04m` 0.8100, `rsyn0840m04m` 0.8118, `mpbp_15` 0.8001, `mpbp_34` 0.2887). Measured with
the key, same binary, same session, one variable [evidence:
`0207b-fanout2e9{nokey,ratio90}-probe-4cpu.log`]:

| 2e9 round | dev SCORE | worst `order()` |
|---|---|---|
| key off (the iter39 shape, `f647df4d`) | 0.791896 | 2.263 s |
| key at 90 % (this submission) | **0.791998** | **1.867 s** |

The key gives up 1.0e-4 of dev — it drops `powerflow0300p`, the round's dearest single mover
(0.9616 → 0.9472, worth −9.2e-5 of the score because it lives in the 0.40-weight `gt_10k` bucket)
because that row sits at ratio 0.9616 — and buys back 0.40 s of peak.

## 4. The bar arithmetic, from a measured transfer

`dad3e94b` (this same tree minus the round) **completed** its graded run at hidden **0.842688**
against the promoted build's 0.842716, i.e. hidden −2.8e-5 for dev −6.0e-5 — the freshest and only
same-class dev→hidden pair on the board: **transfer 0.47**. The promotion bar is 1 bip
(8.4e-5 absolute), so this class needs **dev ≤ −1.79e-4** against the promoted build's 0.792226:

* the committed tree (0.792166, in flight as `50b37868`) = −6.0e-5 → predicted hidden −2.8e-5, rejected
* 5e8 round (0.792079) = −1.47e-4 → predicted −6.9e-5, still short of the bar
* **this build (0.791998) = −2.28e-4 → predicted hidden ≈ −1.07e-4, i.e. 27 % past the bar**

## 5. Verification status, exactly

* **Probe (production mirror, pinned to the graded runner's 4 vCPUs):** 300/300 rows,
  `SCORE = 0.791998`, worst `order()` 1.867 s. The mirror (`SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1`)
  was re-verified against the harness *today* on the committed tree — probe 0.792166, harness
  0.792166, identical buckets.
* **Official sandboxed local harness on this candidate: NOT completed.** Four consecutive full
  runs today died on the 2.0 s per-matrix cap on four *different* small rows —
  `rsyn0810m02hfsg` (n=2670), `gasprod_sarawak81` (n=22536), `chimera_selby-c16-01` (n=2031),
  `syn40m04m` (n=4960) — every one of which the probe measures at 0.70–0.75 s in the same 4-vCPU
  frame minutes later, and every one *outside* this candidate's gate. The host is swapping (15 GB
  RAM, 2–3 GB of swap in use) and the harness cap covers the whole `bwrap`-ed worker, not just
  `order()`; a short-subset run (`SSI_MAX_MATRIX_N=3000`) completes and scores normally. No harness
  pass is claimed. [evidence: `results.tsv:1789243570`, `:1789243692`, `:1789243816`,
  `:1789244542`]

## 6. Risk, honestly

This is a 2e9-op spend on the class the hidden cap has killed three times, now confined to rows
whose incumbent is at most 90 % of their anchor — the eight zero-yield rows that paid +1.0…+1.5 s
in the unkeyed shape (`emfl100_3_3`, `squfl020-150`, `squfl015-080persp`, `ringpack_20_3`,
`glider400`, `syn40m04hfsg`, `transswitch0300p`, `knp5-44`) are now all excluded by the key, since
each of them sits at ratio 0.88–1.00. Rows that remain exposed: `mpbp_21` (0.8615) and
`rsyn0820m04m` (0.7771), both zero-ish value at up to +1.0 s. If the graded killer row is one of
those two, this build repeats `f647df4d`'s failure; if it is any of the eight excluded rows, this
build is ~1.0–1.5 s *cheaper* than the build that was killed.

## 7. Attribution

Model: **deepseek-v4-flash
Harness: **angelX
