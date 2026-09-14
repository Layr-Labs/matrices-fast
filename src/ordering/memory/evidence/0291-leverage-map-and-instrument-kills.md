# iter87 — the score-leverage map, and four measured kills on the zero-value class

All numbers below were produced in this session, in the frames named. Probe frame =
`target/probe-sandbox.sh run` (`cargo test --release -p ssi-candidate-worker`, one
`order()` per row, 24 threads); harness frame = `cargo run --release` (two sandboxed
children per row, 2.0 s cap each). The two frames are **not** the same instrument:
the same tree reads **0.789413 (probe)** and **0.789998 (harness)**.

## 1. The bat resolved: the kill clock moved 2.2×

`4c4f7b7` (fork@600 + chain-displaced registration + 12 sweeps) died on the hidden
per-matrix cap after **184.14 s** (benchmark step `02:56:15.379 → 02:59:19.519Z`),
against six earlier kills clustered at **81.5–87 s**. The tree differs from the 81.5 s
kill `f02eb0d7` by *one pool slot* (`truncate(PEO_ALT_SEEDS + 1)`), so the kill is not a
fixed corpus position reached at a fixed accumulated wall: the same near-cap row set is
flipped by ~0.05 s/row trajectory changes. [.scratch/iter87/remote-4c4f7b71.log:1146]

Promotion bar, read from the repo's own manifest: **`minScoreImprovementBips: 1`** — exactly
1e-4 of hidden score. `3587d1b` (fork@600 alone, completed) missed it by **0.22 bip**.

## 2. The leverage map (new frame: value per row, not wall per row)

`score = Σ_b w_b·gm_b / Σ w_b` with `w = [0.30, 0.30, 0.40]`, so a *relative* improvement
`f` on ONE row of bucket `b` moves the score by `w_b·gm_b/count_b · ln f`:

| bucket | count | gm (dev) | Δscore per 1 % on one row |
|---|---|---|---|
| lt_1k | 147 | 0.8870 | **−0.18 bip** |
| 1k_10k | 108 | 0.8373 | **−0.23 bip** |
| gt_10k | 45 | 0.6817 | **−0.61 bip** |

The `gt_10k` bucket is 2.6× the per-row leverage of `1k_10k`. The dev corpus holds **83
rows at ratio ≥ 0.999** (28 % of the corpus; 94 at ≥ 0.99) — including **five gt_10k
rows at exactly 1.0000** (`kissing2` 20772, `supplychainr1_053050` 16640, `emfl100_5_5`
21925, `emfl050_5_5` 13175, `squfl030-150` 13680). Every one of them extracts *zero*
value, and each 1 % on one of them is 0.61 bip.

## 3. KILL: the anchor gate's justification survives the later gate raises

Four of those five rows pass every class-gate key (`n ≤ 45 000`, `nnz ≤ 200 000`,
`nnz ≤ 16n`, `max_deg ≤ n/2`) and are excluded **only** by `past_anchor = best_flops <
amd_flops`. The iter65 justification for that gate ("the family is never the first
improver on a row") predates the `n`-ceiling / `nnz`-key raises, so it was re-priced with
a new seam (`SSI_ANCHOR_GATE=0`, `src/ordering/mod.rs:5759`):

| arm | rows | ratio | worst order() |
|---|---|---|---|
| shipped | supplychainr1_053050, emfl100_5_5, emfl050_5_5, squfl030-150, knp5-44, squfl020-150 | **all 1.0000** | 0.429 s |
| `SSI_ANCHOR_GATE=0` | same six | **all 1.0000** | 0.490 s |

No row moves; wall rises. The gate is justified even against the current, stronger
exchange. [.scratch/iter87/anchor-control.log, anchor-off.log]

## 4. KILL: the cheap tier is saturated (independent searcher, out of the engine)

An independent Python elimination simulator (validated against the harness's own
numbers: `slay06m` identity 182 109 = 45.2445 × 4 025 ✓, reversed 8 045 = 1.99876 ×
4 025 ✓) ran min-degree, min-fill and 6 randomized-tie min-degree restarts on **all 148
rows with n < 1000**: it beats the engine on **0/148**. Plain min-degree reproduces feral
AMD's exact flops on the control row (4 025, `nnz(L)` 1 113). The lt_1k tier is spent.
[.scratch/iter87/cheap_probe.py, search.py, elim.py]

## 5. Phase census of the cap-critical class

Per-phase seconds, unpinned probe, share of the row:

| row | n | nnz | total | 1.portfolio | 4.subtree | 9.reduce |
|---|---|---|---|---|---|---|
| arki0016 | 7993 | 37 208 | 1.222 | 0.331 (27 %) | 0.220 (18 %) | 0.220 (18 %) |
| gasprod_sarawak16 | 4596 | 15 316 | 1.095 | 0.182 (17 %) | 0.249 (23 %) | 0.126 (12 %) |
| crudeoil_lee4_09 | 15904 | 101 792 | 1.134 | 0.345 (30 %) | 0.155 (14 %) | 0.076 (7 %) |
| pooling_sppa9tp | 5040 | 121 302 | 0.671 | 0.275 (41 %) | 0.229 (34 %) | — |

The three blocks are 45–63 % of every near-cap row; `9.reduce` is the largest
*unattributed* one. [.scratch/iter87/phases-dense.log]

## 6. The `9.reduce` block, priced (new seam `SSI_NO_REDUCE`)

Whole block off, one binary, full 300-row corpus, probe frame:

* control **0.789413** → reduce-off **0.795089**, i.e. the block is worth **−5.7e-3 dev
  (57 bips)** and costs **13.15 s of corpus wall (0.044 s/row)** — by an order of
  magnitude the best value-per-second block in the tree.
* Per-row wall is concentrated exactly on the cap-critical rows (`chp_partload` −0.172 s,
  `transswitch0300p` −0.171 s, `powerflow0300p` −0.161 s, `arki0016` −0.126 s), so it is
  **not** a fence candidate: fencing it buys 0.04 s/row and pays 57 bips.
* Its cheap side is already saturated: the extras run `REDUCE_EXTRA_DEPTHS = [5,4,2,6]`
  under a `REDUCE_WORK_NNZ = 500_000` cap charged as `attempts × nnz`, so on the whole
  `nnz ≤ 60 000` band (4 depths × ≤60 k = 240 k) the cap never binds — there is no
  cap-safe extension left inside the family.
  [.scratch/iter87/reduce-control.log, reduce-off.log, src/ordering/mod.rs:295-333]

## 7. The registration is worth 7.6 bips dev; the seed-slot repair is worth ~0

One binary, new seam `SSI_NO_REG` (registration off) and `SSI_POOL_SLOT`
(`truncate(PEO_ALT_SEEDS + 1)` at the general `flush_batch` site):

| arm | SCORE |
|---|---|
| registration ON, 8 slots (shipped) | **0.789413** |
| registration ON, 9 slots | 0.789420 (+7e-6) |
| registration OFF, 8 slots | 0.790171 (**+7.6e-4**) |
| registration OFF, 9 slots | 0.790197 (+2.6e-5 vs reg-off) |

The registration (the chain-displaced incumbent fed to the donor ledger) is 7.6 bips of
dev value in this frame — the largest single device this lane has measured since the
exchange family. The extra slot is dev-neutral with the registration and *negative*
without it: a pool-slot repair is not a value device.
[.scratch/iter87/noreg.log, reg-slot.log, noreg-slot.log]

## 8. KILL: the fork band cannot be widened into the cheap tier

`SSI_BASIN_FORK_N=1200` (`nnz ≤ 5000` unchanged): dev **0.789407 vs 0.789413 = −6e-6** —
the entire gain is one row (`multiplants_mtg1b` n=645 nnz=4404, 0.6627 → 0.6607) — while
the arm adds +0.05…+0.16 s/row on ~90 rows. The fork's value is entirely inside
`n ≤ 600`; the `(600, 1200]` band is wall-positive and value-free.
[.scratch/iter87/fork1200.log]

## 9. Measured candidate this iteration

The shipped tree + the additive slot at the general ledger site: official local sandboxed
harness **score 0.789998 / fill 0.923120**, buckets 0.886977 / 0.837329 / 0.681764,
**300/300 rows, rc=0, no FAIL**, run wall **284 s**; `cargo test --release -p
ssi-candidate-worker` **126 passed / 0 failed**. Dev-identical to `4c4f7b7` in the
harness frame — the change is a value-monotone ledger repair (consumers install only on a
strict exact decrease) and a trajectory perturbation, not a value device.
[.scratch/iter87/slot-run.log, score.json]
