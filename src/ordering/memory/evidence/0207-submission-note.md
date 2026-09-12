# 0207 — the terminal four-stream round returns at 1/4 budget: the iter40 "removal is free" A/B was a stale-binary artifact

## 1. What changed

One constant, one line of production logic (`src/ordering/mod.rs`):

```rust
- const SHIPPED_ENGINE_FANOUT: &[i64] = &[];               // iter40 "dominated, removed"
+ const SHIPPED_ENGINE_FANOUT: &[i64] = &[500_000_000];   // 4 streams x 5e8 ops
```

Nothing else in the production path is touched. The stage itself (a terminal
`rgreedy::search_par_specs` round over four independent trajectories, merged by a strict
`(flops, source index)` argmin, so its result is a pure function of the four specs and the thread
count cannot change it) is unchanged; only the budget each of its four streams receives changes,
from "never runs" to `5e8` ops per stream. The stage's structural gate
(`n <= 12_000 && (n > 6_000 || nnz > 30_000)`), its fence complement
(`best_flops <= LADDER_FILL_BOUND`) and its four seeds are untouched.

## 2. Why: the recorded justification for the removal does not reproduce

Iter40 removed this stage on the strength of an A/B reported as *byte-identical*:
"`SHIPPED_ENGINE_FANOUT` `&[2e9]` → `&[]` … 0 of 300 dev rows change flops, corpus wall −0.85 s"
with `results.tsv:1789241463` reading `0.791896` with the list emptied. That claim is false on this
tree, and the two independent measurements below say so with different instruments.

**(a) The official sandboxed local harness, production path, no seams, on the *committed* tree
(list emptied), graded 4-vCPU frame** (`taskset -c 0-3`):

```
300 matrices, 0 failures
score 0.792166   tiebreak 0.924495
buckets 0.887462 / 0.838890 / 0.685651
```

[evidence: `0207-harness-tip-4cpu.log`, `score.json`, `results.tsv:1789242561`] That is the
*pre-fan-out* value, not `0.791896`. So the committed tree's own harness number contradicts the
recorded row `1789241463`; the most likely mechanism is a stale `target/release` worker in that
run (a run whose only difference is "identical output, less work" cannot land 2.7e-4 away and be
treated as confirmation of *inertness*).

**(b) A single-variable probe A/B in the graded frame** — one binary, one session, the same three
seams (`SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1`), only the terminal round's budget list changing:

| round budget list | dev SCORE | movers | max per-row +wall | corpus +wall | worst `order()` |
|---|---|---|---|---|---|
| `&[]` (committed) | **0.792166** | 0 | — | — | 1.284 s |
| `&[5e8]` (this submission) | **0.792079** | 12 | +0.365 s (`mpbp_07`) | +6.1 s | 1.416 s |
| `&[1e9]` | 0.792065 | 14 | +0.684 s (`emfl100_3_3`) | +13.6 s | 1.456 s |
| `&[2e9]` (the shape that died remotely) | 0.791896 | 14 | +1.485 s (`emfl100_3_3`) | +23.9 s | 1.812 s |

[evidence: `0207-fanoutoff-probe-4cpu.log`, `0207-fanout5e8-probe-4cpu.log`,
`0207-fanout1e9-probe-4cpu.log`, `0207-fanout2e9-probe-4cpu.log`]

So the terminal four-stream round is worth **2.7e-4 dev at 2e9/stream** and **8.7e-5 at 5e8**, and
the shape that carried it to the grader (`f647df4d`) was the 2e9 point — the one with the largest
price on the class that decides the cap.

## 3. The law this submission is built on: the round's price is linear in its budget and lands on
rows it cannot move

The per-row delta table (same frame, same binary) is sharper than the score:

* at **5e8**: `mpbp_07` +0.365 s (ratio 0.9127 → 0.9127, **zero** gain), `emfl100_3_3` +0.305 s
  (1.0000 → 1.0000, zero), `transswitch0300p` +0.304 s (zero), `squfl020-150` +0.260 s (zero),
  `meanvar-orl400_05_e_7` +0.242 s (zero), `squfl015-080persp` +0.241 s (zero),
  `mpbp_21` +0.237 s (zero), `knp5-44` +0.235 s (zero).
* at **2e9**: the same rows, ~4x dearer — `emfl100_3_3` +1.485 s, `squfl020-150` +1.256 s,
  `glider400` +1.182 s, `rsyn0820m04m` +1.173 s, `syn40m04hfsg` +1.162 s,
  `squfl015-080persp` +1.135 s, `ringpack_20_3` +1.082 s — every one of them **zero** flop gain.
* the 14 rows the round *does* move pay +0.01…+0.27 s each at 5e8 (+0.19…+0.94 s at 2e9), and the
  whole value is concentrated: `rsyn0830m04m` 0.8100 → 0.7854, `powerflow0300p` 0.9616 → 0.9472,
  `rsyn0840m04m`, `mpbp_15`, `mpbp_34`, `pooling_sppa9tp`, `qspp_*`, `qap`, …

That is: the fan-out's 8 dearest rows are rows where the search finds nothing, its price scales
linearly with the budget, and the value it does buy saturates. 5e8 is the best measured
value-per-peak-second point on this curve, and it is the same shape the branch already uses for its
terminal rung ladder ("run only while the row is still being won") — only priced on the budget axis
instead of the ratio axis.

## 4. Verification status, stated exactly

* **Probe (production mirror, pinned to the graded runner's 4 vCPUs, one session):** 300/300 rows,
  `SCORE = 0.792079`, 12 movers, max per-row +0.365 s vs the committed tree's 0.792166.
* **Mirror↔harness correspondence, re-established today on the committed tree:** probe 0.792166,
  harness 0.792166, same buckets. The mirror is `SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1` (the
  `#[cfg(test)]` seams that default to the opposite of production).
* **Official sandboxed local harness on *this* candidate: NOT completed.** Three consecutive full
  runs failed the 2.0 s per-matrix cap today on three *different* rows —
  `rsyn0810m02hfsg` (n=2670, nnz=6964), `gasprod_sarawak81` (n=22536, nnz=75636),
  `chimera_selby-c16-01` (n=2031, nnz=10964) — and the same probe immediately measures those rows
  at **0.70 / 0.75 / 0.72 s** with 3 repeats each, in the same 4-vCPU frame. The host is
  memory-pressured (15 GB RAM, 2 GB of 3 GB swap in use) and the workers run under `bwrap`; the
  harness cap covers the whole worker process, not just `order()`. A short-subset harness run
  (`SSI_MAX_MATRIX_N=3000`) completes and scores normally. So the local cap verdicts are **not**
  usable as evidence about this candidate's spend, and no harness pass is claimed here.
  [evidence: `results.tsv:1789243570`, `:1789243692`, `:1789243816`]
* **Remote:** this is the only channel that counts. The stage has exactly one remote receipt so
  far: `f647df4d` (the 2e9 shape) was killed on the cap. This submission is that shape at 1/4 the
  budget, i.e. ~1/4 of the peak-row price, on the same gate.

## 5. Risk, honestly

`mpbp_07` (+0.365 s, zero gain) is the largest single row increase vs the committed tree; the
2e9 variant's largest was +1.485 s and it died. If the graded killer row is one of the eight
zero-yield rows above, this build pays ~1/4 of what killed `f647df4d` there. Expected hidden
delta: the only measured dev→hidden transfer for this class is 0.66 (dev −2.13e-4 → hidden
−1.41e-4), so −1.47e-4 dev against the promoted build's 0.792226 predicts ≈ −9.7e-5 hidden, i.e.
just past the 1 bip (8.4e-5) promotion bar — a marginal, not comfortable, candidate.

## 6. Attribution

Model: **deepseek-v4-flash
Harness: **angelX
