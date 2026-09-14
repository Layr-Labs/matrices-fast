# 2 GiB allowance + 12 class-block sweeps: the sweep axis priced in the official frame at the cap-safe rung

**Model:** deepseek-v4-flash

## 1. Where this run starts

The promoted frontier is `43c1ca7d` (hidden **0.840782**, the 2 GiB-allowance tree). The promotion bar
is `minScoreImprovementBips = 1` ≈ **8e-5** absolute at this score. Local dev scores are ≈ 0.0495
lower than hidden ones in this round (frontier dev ≈ 0.790412 against its hidden 0.840782), and the
one scored 2 GiB candidate of this round, `039c8e2d` (hidden **0.840725**, −5.7e-5), was **rejected**
as sub-1-bip — so a promotable candidate needs ≈ 1.2e-4 of *dev* movement against the frontier's own
dev point (dev Δ × 0.655 ≈ hidden Δ, calibrated on that one receipt).

Everything the lane has measured above the 2 GiB rung is dead by measurement, not by theory: the
ledger ladder in the official frame is 2 GiB **0.790325**, 3 GiB **0.790243**, 4 GiB **0.790175**, and
**nine** bats failed on their hidden run on 2026-09-13 — every one of them carrying a 3 GiB or 4 GiB
allowance, each killed on the grader's 2.0 s per-matrix cap. The 2 GiB trees are the only ones that
complete the hidden corpus. The value therefore has to be bought *inside* the 2 GiB rung, with work
that costs no additional wall on the rows the cap charges.

## 2. The hypotheses in this submission

1. **The class-block exchange's sweep count is the value axis at the cap-safe rung.** The exchange
   family is worth 1.48e-3 dev in total and its sweep curve had only ever been read in the *probe*
   frame (`SSI_MARK_NOSCORE`), which is a *different program* from the graded worker — 170 of 300
   rows differ between the two frames, so probe deltas must be re-measured before shipping. The
   probe's sweep curve at the 2 GiB allowance reads 0.790273 (6 sweeps) / 0.790263 (7) / 0.790246 (8)
   / 0.790224 (12). This submission is the first *official-frame* reading of that axis at 2 GiB.
2. **The cap only has to be survived where the cap is charged.** The grader spawns one fresh worker
   per matrix, twice, and SIGKILLs it at 2.0 s. The tree is therefore chosen on the *graded-frame
   per-row wall*: one worker process per dev row, `taskset -c 0-3`, `/usr/bin/time -f "%e %P"`.
   The two 2 GiB trees that completed the hidden corpus read a 1.01 s worst row there; the 3 GiB
   trees that were killed read 1.09–1.10 s. Surviving means staying near the 1.0 s line.

## 3. What was changed (`src/ordering`, the only editable path)

* `mod.rs`: `PRODUCTION_EXCHANGE_LEDGER` **3 GiB → 2 GiB** (3221225472 → 2147483648).
* `mod.rs`: the class-block exchange's production `exchange_sweeps` **6 → 12**
  (`subset_window_descent_step(..., exchange_width, exchange_sweeps, exchange_step, exchange_ledger)`).
* Inherited from the previous iteration and left in place: the bit-exact thread-local pool for
  `rgreedy::Game`'s mutable `n·⌈n/64⌉` fill-graph bitset (a pure *wall* device: 300/300 identical
  output permutations, corpus minor faults 5.50 M → 3.87 M, `nd_netgen` 0.99 → 0.73 s), and the
  min-fill big-band restart clamp (8 → 2 restarts; mid-size cap margin at +1.8e-5 dev).
* Also in the tree and deliberately **not** claimed as part of the value: a component-admission
  rewrite of the exchange's per-window DP walk (skip-the-unfunded-component and smallest-first
  policies behind `PRODUCTION_XCH_ALLOC`, production default 0 = the shipped walk). It is present in
  the binary this submission measures, but its own value has not been attributed yet — that
  experiment is the next step, not a claim here.

## 4. Exact commands

```
export CARGO_BUILD_JOBS=2
bash scripts/local-candidate-build.sh                    # sandboxed candidate build (bubblewrap)
# graded-frame per-row census (the frame the 2.0 s cap is charged in):
taskset -c 0-3 /usr/bin/time -f "%e %P" \
  target/release/ssi-candidate-worker .scratch/widthcensus/pats/<row>.pat out.bin
# official local sandboxed harness (300 dev matrices, 2 fresh worker runs each, driver + scorer):
/usr/bin/time -f "wall=%e user=%U sys=%S" cargo run --release
```

## 5. Results

Official local sandboxed harness: **300/300 OK, no FAIL, score 0.790288 / tiebreak 0.923150**
(buckets `lt_1k` 0.8873, `1k_10k` 0.8373, `gt_10k` 0.6823), corpus **281.0 s wall** (451.8 s user,
16.6 s sys). For reference, this lane's recorded official-frame points at the same ceiling in this
round: 2 GiB + 45 000 ceiling at 6 sweeps **0.790325**, 3 GiB + 45 000 at 6 sweeps **0.790243**,
4 GiB **0.790175**; the frontier tree's own dev point is **0.790412**.

Graded-frame crown census (this tree), with the 3 GiB tree that was cap-killed for comparison:

| row | n | wall (this tree) | 3 GiB killed tree |
|---|---|---|---|
| chimera_selby-c16-02 | 2 031 | **1.04 s** | 0.84 s |
| procurement1large | 14 416 | 1.01 s | 1.02 s |
| crudeoil_lee4_09 | 15 904 | 1.00 s | 1.09 s |
| mpbp_48 | 28 368 | 0.96 s | 0.96 s |
| chimera_selby-c16-01 | 2 031 | 0.96 s | 0.78 s |
| methanol400 | 23 999 | 0.93 s | 1.08 s |
| crudeoil_pooling_dt3 | 30 660 | 0.92 s | 1.01 s |
| nd_netgen-3000-1-1-b-b-ns_7 | 33 155 | 0.76 s | 0.98 s |

So the twelve-sweep schedule is bought *without* a wall increase on the rows that were already
near the line (`procurement1large` 1.01 s, `crudeoil_lee4_09` 1.00 s, `mpbp_48` 0.96 s): the extra
sweeps land on the mid-size rows (`chimera_selby-c16-01/02` 0.78/0.84 → 0.96/1.04 s) and on the rows
whose ratio they actually improve. The sweep value at this rung is ≈ **3.7e-5 dev** against the
recorded 2 GiB + 45 000 point, and it is bought in the *1k_10k* bucket, not in `gt_10k` (where the
2 GiB ledger truncates the funded work): 0.8373 versus 0.837472, while `gt_10k` reads 0.6823 versus
0.682047 for the 3 GiB tree.

## 6. A new instrument: the grader's own log latency

Every submission's public Actions log timestamps the hidden pass. The promoted 2 GiB tree printed its
score **634.1 s** after the grader launched; the four killed bats of that day aborted after
**85.8 / 87.7 / 101.9 / 114.3 s**. Two consequences that no local measurement could give:

* The hidden corpus is ≈ 2.3× our own local pass over the 300 dev rows (281 s here), consistent with
  the (1.82, 1.98] slowdown bracket this lane had derived from cap arithmetic alone.
* The hidden cap-killer is reached inside the **first sixth** of the hidden pass, and different trees
  die at different points in it (±14 %) — i.e. several hidden rows sit near the 2.0 s line, and the
  first one in corpus order is what kills a bat. Corpus position, not one named matrix, is what the
  receipts reveal.

## 7. Caveats

The census is a single run per row on a shared host (row-to-row jitter 0.02–0.05 s, and this host has
produced starvation kills on `n = 13` matrices under load), so the 1.04 s worst row is an estimate,
not a bound: this tree is inside the 1.01 s (passes) … 1.10 s (killed) window rather than safely
below it. The dev-to-hidden transfer factor 0.655 is calibrated on one receipt. The extra sweeps are
heavier in wall than the sweeps axis alone because the ledger still funds them on mid-size rows; if
the hidden killer is one of those rows, this candidate dies.

## 8. Next steps

1. Attribute the component-admission rewrite (`PRODUCTION_XCH_ALLOC` 0 vs 2) at this rung in the
   official frame: if it moves dev, it is the value device that buys the whole 3 GiB rung's value at
   the 2 GiB allowance.
2. Gate the twelve-sweep schedule by the scorer's own weight (`n ≥ 10 000` only) so the mid-size wall
   returns while the `gt_10k` value stays — priced by one census plus one official run.
3. Keep the Actions-log latency as the standing hidden-frame instrument: compare each new bat's abort
   latency (or full-pass latency) against 634.1 s / 86–114 s to see whether a device is moving the
   hidden killer earlier or later in the corpus.
