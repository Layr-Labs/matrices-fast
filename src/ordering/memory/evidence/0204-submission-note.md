# 0204 — the terminal ladder becomes a **live-winning device** (ratio gate at 90 %)

## 1. Context and goal

The frontier on `layr-labs/matrices-fast` is this solver's own promoted build
`4e45ee63` (hidden `0.842716`, `geomean_fill_ratio 0.945064`), which is the tree
that carries: the fill fence (`fill > 2e10 -> keep 64 candidates per batch`,
stride kept), the terminal draw window `n <= 10 000` (`SHIPPED_FULL_N`), the
stage-1b force arm, and a **single** 2e8 `rgreedy` rung on the dense band.
Promotion needs a hidden delta better than `0.842631` (>= 1 bip = 8.4e-5
absolute at this frontier).

The previous submission from this line, `cad51a0f` (dev `0.792110`), was exactly
this tree plus three extra dense-band rungs (`[2e8, 1e8, 1e8, 1e8]`) and it came
back **failed on the 2.0 s per-matrix cap**:

```
cad51a0f-1173-429f-ae07-c14c5d6092ea   failed   gh run 34708974859
Benchmark step 17:45:00.31 -> 17:46:51.90 = 111.6 s
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

while its single-rung parent `4e45ee63` ran the Benchmark step 525 s to
completion and promoted. So: **the four-rung add is worth 1.16e-4 of dev flops
and is not survivable as-is**. The goal of this iteration was to find out *why*
it is not survivable — with per-row evidence rather than a class label — and to
ship the exchange that keeps its value while deleting its risk.

## 2. Environment, setup, instruments

* Repo: `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`,
  Rust toolchain pinned by `rust-toolchain`, sandboxed harness
  (`scripts/local-candidate-build.sh` + `cargo run --release`, the same binary the
  grader dispatches, `TIME_CAP_PER_MATRIX = 2 s` in `src/main.rs`).
* Probe (mirrors production through three `cfg(test)` seams whose defaults are
  the opposite of production — `indep_force_off`, `sparse_large_tie_on`, fence
  stride mode):

  ```sh
  SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1 SSI_NO_SPARSE_LARGE_TIE= \
    cargo test --release -p ssi-candidate-worker -- --ignored --nocapture probe_timing_and_score
  ```

  This reproduces the graded local harness exactly (0.792226 / 0.8874 / 0.8391 /
  0.6857), so every A/B below is one binary, one host, one session.
* New in this iteration: a `FENCETRACE` instrument (test-only) that prints every
  candidate batch's size, its incumbent fill and both fence predicate values, and
  a `RATIOGATE` trace line that prints the row's live ratio-to-anchor and whether
  the rung list ran.

## 3. What was measured first (and what it ruled out)

**(a) The fence is blind to the corpus's real heavy tail.** Dev's three giants
carry incumbent fills of `3.5e7 / 1.4e9 / 3.2e9` (`acopf_case9241pegase_qcqp`
n=313 068, `gabriel10` n=244 056, `faclay75` n=272 878) against a fence bound of
`2e10`, and they are among the corpus's slowest rows (1.13 / 1.02 / 0.82 s of a
116 s corpus). A fence keyed on fill cannot see them. I implemented a second,
cost-keyed predicate (`nnz >= K`, with an `SSI_LADDER_COST_NNZ` seam) to price
that idea — and then **killed it with the same instrument**: `FENCETRACE` shows
the real giants run **1-3 candidate tasks per batch** (`faclay75` 1, `acopf` 2,
`gabriel10` 3+2), because the restart budget is `budget / nnz` and the queue is
long only on *small* matrices (527 tasks on n = 30..120 rows). There is simply
nothing on those rows for a batch fence to truncate, so a cost key buys no cap
margin where the giants are. The residual cost-key device is left in the tree as
`LADDER_COST_NNZ = usize::MAX` (off) with the seam intact, so the finding is
reproducible rather than lost.

**(b) The pipeline's per-row time is a *plateau*, not a size law.** On the dev
corpus the per-row time is 0.83-1.19 s for the twenty slowest rows whose n spans
7 993 to 313 068 and nnz spans 37 208 to 1 379 706 — i.e. ~25 stages each costing
10-240 ms add up to a size-flat ~1 s. Phase marks (`SSI_PROBE_PHASES`) confirm:
`faclay75` spends 0.182 s in `1.portfolio`, 0.196 s in `1b.indep`, 0.097 s in
`9.reduce`, and gains **zero** flops after `1.portfolio` (0.9405 -> 0.9405);
`kissing2` spends 0.42 s and never leaves 1.0000. So the cap is decided by a sum
of many small gates, not by one spender — which is why a single fence has never
been able to bound it.

**(c) An "early stop on near-baseline rows" is NOT free** (hypothesis killed):
cutting the pipeline after `1b.indep` for rows whose live ratio is >= 0.98 costs
`+0.00297` dev (98 rows, saving 19.8 s); after `4.subtree` at >= 0.98 it costs
`+0.00191` for 8.4 s. Rows that look hopeless at ratio ~1.0 do get rescued later
(`nuclear104` 1.0000 after the portfolio -> 0.7593 final), so a static
stop-at-baseline rule would have deleted real value.

**(d) The killed add's price, per row.** One binary, two trees
(`0204-r1-control-fencetrace.log` = four rungs, `0204-full-phases.log` = frontier
single rung, same session), diffed per row:

| row | n | frontier | 4 rungs | delta | ratio change |
|---|---|---|---|---|---|
| `arki0016` | 7 993 | 0.888 s | 0.987 s | **+0.099** | none (0.9512) |
| `mpbp_15` | 9 858 | 0.806 s | 0.914 s | **+0.108** | none |
| `mpbp_07` | 9 532 | 0.856 s | 0.929 s | **+0.073** | none (0.9127) |
| `powerflow0300p` | 11 251 | 0.857 s | 0.913 s | +0.056 | none |
| `wastewater05m1` | 98 | 0.329 s | 0.442 s | +0.113 | -0.0026 |
| `nuclear25a` | 1 942 | 0.428 s | 0.450 s | +0.022 | -0.0118 |

187 rows changed by > 0.005 s, +10.7 s in total; **only 10 rows changed value**,
and every one of them was already at ratio <= 0.86 when the rungs ran. The rows
that took the largest additions in the *cap-relevant* class were all at ratio
0.91-0.96 with zero flop gain. That is the mechanism the kill receipt was missing:
the rungs were paying 0.03-0.11 s per row for a device that only pays on rows the
pipeline is already winning.

## 4. The change

1. `SHIPPED_LADDER_EXTRA` (four-rung dense ladder) is restored.
2. `const LADDER_RATIO_PCT: u64 = 90;` — the terminal rung list runs only while
   the row's own incumbent is at most 90 % of `anchor_flops`, a snapshot taken
   once after the first scored batch (the AMD floor, possibly already improved by
   the AMD/AMF variants in that batch). Below the gate the ladder is `Vec::new()`,
   so the change removes *the shipped rung too* on those rows.

The key is a **posterior** quantity — the row's own live result — not an
`n`/`nnz`/fill window fitted to any instance, and it is scale-free: no constant
in the gate is derived from a dev row's identity. Test seam
`SSI_LADDER_RATIO_PCT` (0 disables), production constant 90.

## 5. Results (all one binary, probe mirroring production)

| build | SCORE | Σ `order()` | `arki0016` | `mpbp_07` | `mpbp_15` | `wastewater05m1` |
|---|---|---|---|---|---|---|
| frontier (1 rung) | 0.792226 | 125.0 s | 0.888 | 0.856 | 0.806 | 0.329 |
| `cad51a0f` (4 rungs, killed) | 0.792110 | 135.6 s | 0.987 | 0.929 | 0.914 | 0.442 |
| **this build** | **0.792166** | **115.8 s** | **0.839** | **0.743** | 0.853 | 0.439 |

* `SSI_LADDER_RATIO_PCT=100` reproduces the frontier's `0.792226` exactly — the
  gate's wiring is neutral when it cannot fire.
* Threshold sweep with the four rungs: 90 % -> 0.792166, 85 % -> 0.792166 (same
  to 6 dp, Σ 115.8 vs 115.3 s); 90 % is shipped as the more conservative end.
* 209 of the 263 traced in-window rows are closed by the gate, i.e. the ladder
  is now pure spend nowhere except rows it is moving.
* Largest per-row **increase** against the frontier anywhere in the corpus:
  +0.110 s on `wastewater05m1` (a 0.44 s row); among rows >= 0.5 s the largest
  increase is **+0.077 s** (`mpbp_47`, 0.666 -> 0.743 s, ratio 0.80) and the
  *sum* of deltas over those 94 rows is **-1.99 s**.

## 6. Official sandboxed local harness (production path, no seams)

```sh
bash scripts/local-candidate-build.sh && cargo run --release
```

```
1789237250  OK  0.792166  0.924495
lt_1k    147  0.887462  0.959840
1k_10k   108  0.838890  0.946895
gt_10k    45  0.685651  0.881186
score 0.792166   tiebreak 0.924495   (300 matrices)
```

against the promoted build's `0.792226 / 0.924450`: **-6.0e-5 flops**,
+4.5e-5 fill, `gt_10k` bucket unchanged, corpus wall time -7.4 %.

## 7. Tradeoffs, caveats, failures, next steps

* The ungated add cost +10.7 s of corpus time; the gated pair costs **-9.2 s**
  relative to the frontier while still buying part of the value, so this is a
  removal-shaped submission with a real flop claim, not a cap gamble.
* Caveat: per-row wall times on this host repeat to +-0.05 s (the same A/B shows
  `acopf` -0.060 s and `gabriel10` +0.009 s on rows the change cannot touch), so
  only deltas above ~0.06 s and the deterministic SCORE are load-bearing. The
  deltas quoted for `arki0016` / `mpbp_07` / `mpbp_15` exceed that floor.
* Caveat: the gate closes the ladder on rows whose anchor was already strong
  (`anchor_flops` includes the AMD/AMF variants of the first batch), which is a
  slightly stricter key than "ratio vs AMD". It was chosen because it needs no
  extra baseline computation inside `order()`.
* Killed this iteration (recorded so it is not retried blind): a cost-keyed
  batch fence (nothing to truncate on the real giants), and a static
  stop-at-near-baseline rule (late stages rescue rows at ratio 1.0).
* Next: the same live-ratio key is the natural place to admit *more* rungs
  (a fifth rung was worth 6e-6 ungated); the open question is whether the hidden
  corpus's fenced rows carry any tail value at all, which the `7bba8604`
  receipt (kept set at constant count, failed) still leaves open.
