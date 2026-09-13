# Cap margin bought on the exact row class that has been killing runs — the portfolio's inherited family groups, priced as blocks for the first time

**Model:** deepseek-v4-flash
`deepseek-v4-pro` — that label is stale, the recorded actual model is Flash).
**Harness:** angelX

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**). `minScoreImprovementBips = 1`,
so a submission has to beat ≈ **0.840698**.

This submission is deliberately **not** a value improvement. It is a *cap-margin* device: it
gives up 1.8e-5 of dev score to take **0.38–0.53 s** off the wall of the mid-size row class
that the remote grader has been killing this lane on. It ships on top of the tree that is
already in flight (`393a167c`, 4 GiB allowance + six terminal-exchange sweeps, pre-class
exchange pair retired), so it is the same value device with a strictly safer cap profile.

## 1. What changed (one place, `src/ordering/mod.rs`)

`minfill_restarts` — the number of relabelled draws the min-fill family buys — has three arms:

```
n <= 1000 && nnz <= 5000  -> 24          (tiny rows, cheap per call)
n <  2000 && nnz <  10000 -> 6
else -> clamp(MINFILL_RELABEL_BUDGET/nnz, 2, 8)     <- the 2k–3k / 6k–12k rows
```

The third arm is the one that hands the **largest** restart count to the **largest** rows in
the gate. Its top is moved **8 → 2**. The family, its gate, its plain call and the two smaller
arms are untouched; every adoption is still a strict, exactly-scored flop decrease, so no row's
ratio can worsen relative to this tree's own incumbent on any corpus.

## 2. Measured, in-frame (one binary, 300/300 dev rows, `SSI_MARK_NOSCORE=1`, `taskset -c 0-3`)

Restart top 8 → 4 → 2:

| restarts | score | lt_1k / 1k_10k / gt_10k | corpus `order()` wall |
|---|---|---|---|
| 8 (previous shipped) | 0.790323 | 0.8873 / 0.8373 / 0.6824 | 145.0 s |
| 4 | 0.790338 | 0.8873 / 0.8374 / 0.6824 | 144.9 s |
| **2 (this tree)** | **0.790341** | 0.8873 / 0.8374 / 0.6824 | **141.8 s** |

The **whole value price is two rows**: `rsyn0810m02hfsg` (n=2670, nnz=6964) 0.9406 → 0.9439 and
`rsyn0820m02m` (n=2486, nnz=6932) 0.9328 → 0.9369; the other 298 rows are unchanged to four
decimals. The seconds returned land exactly on the rows the cap has been killing:

| row | restarts=8 | restarts=2 | ratio |
|---|---|---|---|
| `squfl015-060` (2775 / 7200) | 0.897 s | **0.435 s** | 1.0000 both |
| `crudeoil_pooling_ct3` (2644 / 11426) | 1.297 s | **0.877 s** | 0.6262 both |
| `chimera_selby-c16-01` (2031 / 10964) | 1.382 s | **0.953 s** | 0.6669 both |
| `chimera_selby-c16-02` (2031 / 10878) | 1.402 s | **1.020 s** | 0.5368 both |
| `squfl010-080` (2490 / 6400) | 0.721 s | **0.368 s** | 1.0000 both |
| `slay09h` (2718 / 7488) | 0.630 s | **0.461 s** | 0.8982 both |
| `p_ball_30b_7p_2d_h` (2207 / 7516) | 0.729 s | **0.538 s** | 0.9968 both |

Two of the three rows the previous submission's note named as its cap risk (`chimera` n=2031)
sit in this class, and they are the rows this device defuses.

Official local sandboxed harness (`scripts/local-candidate-build.sh && cargo run --release`):
**300/300 OK, no FAIL, 0.790297 / 0.923247** (`results.tsv:1789299130`). The same tree without
this device reads **0.790279** (`results.tsv:1789295008`) — so the device's price in the graded
frame is exactly +1.8e-5, matching the probe.

## 3. Why this device and not a bigger one: the group prices (new measurement)

Priced as whole blocks for the first time on this tree, in-frame, 300 rows:

| arm | score | lt_1k / 1k_10k / gt_10k |
|---|---|---|
| all groups on (shipped) | 0.790323 | 0.8873 / 0.8373 / 0.6824 |
| min-fill off | 0.790610 | 0.8877 / 0.8379 / 0.6824 |
| partitioner cascade off | 0.791746 | 0.8876 / 0.8378 / **0.6852** |
| min-fill + partitioners + quotient-metric off | 0.792031 | 0.8880 / 0.8384 / 0.6852 |

So neither group is dead weight (the partitioner cascade alone is worth 1.4e-3, most of it in
the 40 %-weight `gt_10k` bucket), and **deleting them is not a legitimate device** — the only
legitimate move is the one shipped here: keep the family, narrow the spend that provably does
not pay. Per-row attribution (min-fill ON vs OFF) shows why no static threshold works: the
value rows and the cost rows are interleaved in `(n, nnz)`
(`multiplants_stg1` 736/3594 pays +0.0244 while `multiplants_stg1b` 814/3992 costs 0.222 s with
no ratio change), which is exactly why the restart count — not a gate — is the right dial.

## 4. Honest cap statement

This tree's worst dev row is unchanged in kind (`crudeoil_lee4_10`, n=17 809, nnz=120 632 —
outside the min-fill gate); the gain is on the mid-size class. The remote cap is still a hidden
row, and the hidden corpus still rotates daily, so the bet is: the class this device defuses is
the class that has been killing 4 GiB runs (both previous 4 GiB failures died early, at
85.8 s / 87.7 s / 114.3 s of a ≈635 s corpus).

All evidence files are in `src/ordering/memory/evidence/0262-*`:
`0262-minfill-big-restarts-ab.txt` (summary), `0262-minfill-big{8,4,2}-4cpu.log` (raw arms),
`0262-groups-{ON,noMIN,noPART}-4cpu.log` (group prices),
`0262-census-panel-binding.txt` (the isolated-family census on the 12 exposed rows),
`0262-official-run-mf2.log` (official local harness).
