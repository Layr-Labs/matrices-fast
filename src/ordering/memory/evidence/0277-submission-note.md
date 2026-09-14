# iter77 — the dense-band second ladder rung (and the at-floor class closed)

Model: **deepseek-v4-flash** (the live run identity the harness stamps), harness **angelX**.
Editable path touched: `src/ordering` only (`mod.rs`, `probe.rs`, plus the local evidence
ledger under `src/ordering/memory/`).

Local score of this tree, official sandboxed harness
(`bash scripts/local-candidate-build.sh && cargo run --release`):
**0.790253 / 0.923133, 300/300 rows, no FAIL, 126 tests pass**
(the tree this replaces reads **0.790277 / 0.923142** on the same box — the iter75
charge-shape tree, which read 0.790282 before it).

## 1. What changed in production

**One device: a second rung on the terminal exact-objective engine ladder, gated to the
dense band.** The ladder (`SHIPPED_LADDER`, budget 2e8 = one draw, `nnz < 3n` rows get 50M)
now becomes two rungs `[(200e6, seed0), (500e6, seed1)]` when **`nnz >= 10n`** — the same
structural dense band the crown already uses — inside the existing sparse/else fork. There is
no matrix identity anywhere in the gate: it is two structural predicates (`nnz >= 10n`, and
the pre-existing `n <= 10 000` window + `nnz <= 200 000` fence that already bound the ladder).

Probe-side additions (test-only, never compiled into the shipped worker), all recorded for
the ledger: `probe_floor_certificate`, the `SSI_PART_FLOOR` seam, and `SSI_TERM_DENSE_RUNG`
(the A/B switch for this device).

## 2. Why the dense band, and not the whole class — the measurement

I walked every dev row the ladder already searches (`n <= 10 000`) and stepped the engine's
budget, re-seeding `rgreedy::search` from the shipped incumbent (`probe_engine_census`):

| budget rung | wins on the 29-row class | corpus wall |
|---|---|---|
| 2e8 (the shipped rung) | **0** | — |
| 5e8 | 4 (`mpbp_15`, `mpbp_07`, `pooling_sppa9tp`, `qspp_0_14`) | +2.4 s |
| 1e9 | +2 (`crudeoil_lee2_06`, `mpbp_15`) | +7.5 s, **with two regressions** |
| 2e9 | 0 more | +4.9 s |

The 1e9 arm's regressions (`rsyn0840m04m` 0.8040 -> 0.8056, `meanvar-orl400_05_e_7`
0.9998 -> 0.9999) are a real property of this tree, not noise: the draws and the class
exchange draw on a **shared charge ledger**, so a bigger draw spends budget the exchange
would have used. That is why the shipped rung is 5e8 and not larger.

The winners split cleanly by density, and so does the *price*:

* dense winners (`pooling_sppa9tp` nnz/n = 24.1, -0.68 %; `pooling_digabel19` 10.4,
  -0.33 %; `qspp_0_14` 214x, -0.11 %; `qspp_0_13` 184x) pay on rows with **0.55 s or more
  of slack** (the slowest dense-band row is `pooling_sppa0pq` at 0.733 s);
* every non-dense winner (`mpbp_15` +0.104 s on a 1.137 s row, `mpbp_07` +0.098 s on 1.062 s,
  `crudeoil_lee2_06` +0.059 s on 1.116 s) lands on exactly the row profile that has killed
  nine submissions today, so it is *excluded by construction*, not by tuning.

Value and wall, measured:

* **value, worker frame**: 0.790277 -> **0.790253** (-2.4e-5 dev). Movers, identical in the
  probe frame and in the official per-row table: `pooling_digabel19` 0.8433 -> 0.8405,
  `pooling_sppa9tp` 0.16134 -> 0.16021, `qspp_0_14` 0.99498 -> 0.99367, `qspp_0_13`
  0.99480 -> 0.99460. (The probe's own delta is -2.85e-5, so unlike the iter75 charge shape
  this device transfers across the two frames.)
* **wall, min of 2 reps per arm over all 33 dense-band rows**: +1.92 s corpus-wide; the four
  dearest rows are `torsion50` 0.582 -> 0.796 s, `graphpart_clique-20` 0.349 -> 0.533,
  `st_m1` 0.247 -> 0.425, `knp5-44` 0.235 -> 0.365 — all far below this frame's worst row
  (1.33 s) and below the ~1.28 s 4-core survival line this lane has derived. **No census or
  crown row is in the band**: `arki0016` (nnz/n = 4.7), `chp_partload` (3.2),
  `rsyn0840m04m` (1.1), `transswitch0300p`, `chimera_selby-c16-01`, `mpbp_48`, `lee4_09/10`
  (n > 10 000) are all outside it, and the crown-rank deltas I first saw were host noise,
  which is why the wall table is reported as a min-of-2 A/B.

## 3. Two things this iteration closed with evidence (so nobody re-prices them)

1. **The at-floor class is not upside.** 75 of 300 rows end at ratio *exactly* 1.0000. A new
   probe shows **16 of them are certified zero-fill** — `order()` returns AMD immediately when
   `nnz_l == n + edges`, and a fill-free ordering attains the graph's flop lower bound, so
   those 16 rows are at the global optimum: no submission can move them. The other 59 were
   probed to exhaustion: the exact-objective engine returns **no candidate at all** on the 13
   hand-picked at-floor rows even at 4e9/2e10 word-ops (and 1e11 where affordable), a new
   `SSI_PART_FLOOR=1` seam that widens the tuned/hi-trial/shape-variant partitioner gate
   (which is structurally OFF on those rows) moves **no ratio**, and the iter34 engine battery
   found no library engine win either.
2. **The pipeline's bookkeeping is still clean.** `probe_eval_audit` re-run on this tree
   (first re-run since the instrument was written): 0 leak rows over 3122 scored candidates —
   no permutation the pipeline scored was left unshipped.

Side measurement with a bearing on the next device class: the cap-critical rows are **serial**
(`RAYON_NUM_THREADS=1` vs `4`, `SSI_MARK_NOSCORE=1`: `chp_partload` 1.2465/1.2690 s,
`arki0016` 1.2310/1.2657, `lee4_09` 1.1392/1.1454 — within 2 %), so thread parallelism buys
nothing on the rows that bind the cap today, and our own corpus cannot decide whether the
hidden frame charges wall or CPU because these rows are serial either way.

## 4. Honest caveats

* The hidden frame is ~1.53-1.56x slower per row than this box and its rows are heavier; the
  +0.2 s this rung adds to the dearest dense rows is 4-6x below the survival line *here*, but
  I cannot see the hidden corpus, and every submission of mine that added per-row work in the
  10 000 < n <= 50 000 band has been cap-killed. That is precisely why the rung is gated to a
  band whose dev members carry 0.55 s+ of slack and whose gate excludes every crown row.
* Value is -2.4e-5 dev, i.e. below the ~1.29e-4 dev that a promotion needs under this lane's
  measured 4.3x dev->hidden transfer for the sweep device. If the transfer for this family is
  smaller, this is a margin-neutral value bat rather than a promotion.

Evidence bundle: `src/ordering/memory/evidence/0277-dense-rung.txt` plus
`0277-engine-census-10k.log`, `0277-ab-ladder-*.tsv`, `0277-wall-dense-*.tsv`,
`0277-floor-certificate.log`, `0277-lns-beyond-gate.log`, `0277-eval-audit.log`,
`0277-probe-dense-*.log`, `0277-official-run-dense-rung.log`.
