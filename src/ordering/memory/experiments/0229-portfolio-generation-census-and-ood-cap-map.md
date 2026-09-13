# 0229 — portfolio generation census + the out-of-distribution cap map

## Questions

1. **What is `1.portfolio` actually spending?** It is the largest single spender on
   the rows where the 2 s cap binds (`crudeoil_lee4_10` 0.48/1.23 s,
   `chimera_selby-c16-02` 0.76/1.38 s) and nothing in the record separated
   *producer* work from *exact scoring* work, or counted how many orderings are
   scored per row and how many of them ever lower the running minimum.
2. **Where is the cap actually threatened?** Every cap receipt in the record is
   either a dev row (peak 1.25 s) or an unexplained remote FAIL. The seven
   structural stress corpora built in 0196/0201 were never run against the
   current tree with per-stage attribution.

## Instrument (test-only, `#[cfg(test)]` only — never in the shipped worker)

- `parallel::stats` + counters inside `parallel::run_generic`'s `eval`:
  generator wall time, exact-scorer wall time, producers, scores paid, memo
  hits, and the split of scoring time into candidates that did/did not lower
  the running minimum.
- `portstats_line(stage, n, nnz, queued)` prints those counters plus the number
  of a block's queued producers (`tasks.len()`) at 12 boundaries inside
  `leader_order`, so one line per (row, block) with `SSI_PORTSTATS=1`.
- Reruns: `target/probe-sandbox.sh` forwards `SSI_CORPUS_FILE`, so the same
  probe binary runs the dev corpus, `/tmp/scale`, or `/tmp/ood` unchanged.
- All timings below are `taskset -c 0-3` (the grader's 4-vCPU width), most with
  `SSI_MARK_NOSCORE=1` so the phase marks do not add 25 scoring passes.

## Findings

1. **Generation, not scoring, is the portfolio's cost.** dev peak rows, CPU-summed
   over `PAR_MAX_THREADS`: `chimera_selby-c16-02` (n=2031) gen 2.84 s vs score
   0.125 s; `crudeoil_lee4_10` (n=17809) gen 1.52 s vs score 0.117 s;
   `crudeoil_pooling_ct3` (n=2644) gen 2.75 s vs 0.124 s. **12-25x**. A perfect
   scorer-side filter could therefore save at most ~5 % of the stage.
2. **The candidate count is an `n`-window schedule, not a density schedule.**
   574 producers on n=2031/2644, 334 on n=4799, 388 on n=6028, 207 on n=7993,
   116 on n=10429, 85 on n=15904/17809, 8 on n=39098, 2-5 above n=240k. The
   per-block queue profile is identical for rows with identical (n, nnz)-window
   membership (`chimera_selby-c16-02` and `crudeoil_pooling_ct3` both queue
   103/84/84/60/2/48/134 by block). [0229-blockstats-peakrows-4cpu.log]
3. **93-99 % of the scored candidates never lower the running minimum**, and the
   tail's yield is structure-dependent: on dev, the last flush (METIS shapes +
   relabelled-AMF + heavy tier) scores 547 candidates on n=2031 for 5 wins, but
   193 candidates on `arki0016` (n=7993) for **0 wins** at 1.66 s of CPU.
   [0229-blockstats-peakrows-4cpu.log]
4. **The dev peak is a fitted property, not a global bound.** Running the current
   shipped tree over the 27-row `/tmp/ood` structural corpus: **7 rows exceed the
   2 s cap** — `ood_kkt_b200x200+400` 11.08 s, `ood_ba_m6_n40000` 6.57 s,
   `ood_kkt_b100x300+300` 4.21 s, `ood_rand_d8_n60000` 3.62 s,
   `ood_rand_d6_n300000` 3.57 s, `ood_rand_d40_n20000` 2.30 s, and
   `ood_kkt_b40x200+200` (n=8200, nnz=143754 — *inside* the exchange block's
   n ≤ 12 000 band) 2.19 s. [0229-ood-capmap-4cpu.log]
5. **On the over-cap rows the portfolio's cost is a handful of very expensive
   producers**, not a large candidate list: `ood_kkt_b200x200+400` scores 4
   candidates and burns **6.27 s of generator CPU in the first batch** (3
   producers) plus 2.02 s for the 4th; its whole pipeline is
   `1.portfolio` 4.46 s + `1b.indep` 1.94 s + `9.reduce` 3.43 s = 9.8 of 11.2 s.
   [0229-kkt40400-phases-4cpu.log]
6. **The smallest structural row over the cap is owned by three stages**:
   `ood_kkt_b40x200+200` 2.014 s = `1.portfolio` 0.828 s + `1b.indep` 0.797 s +
   `9.reduce` 0.158 s + tail. In that row's final flush, **26 of 41 producers
   (METIS shapes 12 + relabelled-AMF 12 + AMF-extra 2) cost 2.08 s of generator
   CPU and lower the running minimum zero times** (the `wins` counter is 1
   before and after; the portfolio's final ratio 0.9807 is the first batch's).
   [0229-kkt8200-phases-4cpu.log]
7. **A fixed-`n` density sweep (n = 8000-9950, nnz/n = 10 → 28) does not blow the
   cap**: worst 1.44 s over 18 rows, and the producer count *falls* with density
   (81 → 40 → 23 → 19 → 16) because the candidate cache's key includes the
   dense-deferred set, so higher density collapses α-variants into one key. The
   OOD danger is the *block-angular / scale-free* structure, not density as
   such. [0229-density-scale-4cpu.log]
8. **In-band cost of the in-flight device is ≤ 0.035 s.** A/B of the exchange
   window shape 12/4/5 (in flight) against the promoted 8/4/3 over the 18-row
   in-band scale corpus, one binary, `SSI_PROBE_REPEAT=2`, 4 vCPU: max |Δt| =
   0.035 s, and the wider window is *better* on all 6 rows whose flops move
   (e.g. 0.9956 → 0.9962 backwards for 8/4/3). No row in the band regresses.
   [0229-scale-xchg12-4cpu.log, 0229-scale-xchg8-4cpu.log]

## Consequence

The cap-risk axis of this pipeline is *structural*, not dev-shaped: the grader's
dev corpus has no row of the block-angular / scale-free family that costs 2-11 s
here, so a dev-only peak reading cannot certify a device. Two of the three
stages that own the over-cap rows (`1b.indep` at 40 % of the n=8200 row) are
charged *before* any acceptance gate, and the third (`9.reduce`) has no dev
counterpart at all. Any future "buy cap margin" device should be priced on this
corpus, and the in-flight exchange shape itself is clean in-band (finding 8).

## Reproduce

```sh
bash target/probe-sandbox.sh build
SSI_PROBE_ONLY=<rows> SSI_PROBE_PHASES=1 SSI_MARK_NOSCORE=1 SSI_PORTSTATS=1 \
  taskset -c 0-3 bash target/probe-sandbox.sh run
BIN=target/probe/release/deps/ssi_candidate_worker-*   # direct, for /tmp corpora
SSI_CORPUS_FILE=/tmp/ood/patterns.jsonl SSI_PORTSTATS=1 SSI_MARK_NOSCORE=1 \
  SSI_PROBE_PHASES=1 taskset -c 0-3 $BIN --ignored --nocapture \
  --test-threads=1 probe_timing_and_score
```

## 0229b — the parity pass priced on structural rows (why it died remotely)

Same binary, same session, `SSI_PROBE_REPEAT=2`, 4 vCPU, `/tmp/ood` rows, with
and without `SSI_INDEP_PARITY=1` (the device that shipped as `41baf1ce` /
`ab5adbbd` and failed remotely twice):

| row | n | parity off | parity on | Δ | ratio off → on |
|---|---|---|---|---|---|
| `ood_grid3d_16` | 4096 | 1.0842 s | **1.3165 s** | **+0.232 s (+21 %)** | 0.7341 → 0.7292 |
| `ood_kkt_b40x200+200` | 8200 | 2.0067 s | 2.1174 s | +0.111 s | 0.9599 → 0.9599 |
| `ood_rand_d12_n6000` | 6000 | 1.1951 s | 1.2484 s | +0.053 s | 0.9851 → 0.9851 |
| `ood_grid2d_40` | 1600 | 0.7574 s | 0.9191 s | +0.162 s | 0.7754 → **0.7763 (worse)** |
| `ood_rand_d12_n2000` | 2000 | 0.9722 s | 0.9580 s | −0.014 s | 0.9745 → 0.9745 |
| `ood_grid2d_80` | 6400 | 0.8541 s | 0.8533 s | −0.001 s | 0.6321 → 0.6321 |
| `ood_grid3d_26` | 17576 | 0.6909 s | 0.6855 s | −0.005 s | 0.6598 → 0.6598 |

Three rows fired the parity rule (PARITY trace). Two conclusions:

1. **The kill mechanism is measured, not inferred.** Where the rule fires, the
   extra `2.descent + 3.search + 4.subtree` pass costs 5-21 % of the row's own
   `order()` — a 1.08 s structural row becomes 1.32 s, and a row already at the
   cap (2.0 s) stays over it. Dev contains no row of this structure, which is why
   two 300/300 local receipts could not see it.
2. **Parity's value on structure is mixed, not monotone**: +0.7 % on
   `ood_grid3d_16` but −0.09 % (worse) on `ood_grid2d_40` — the second measured
   instance of the entry-dependence already recorded for `wastewater05m1`
   (the post-4b stages invert the 4b comparison), this time on a row the dev
   corpus cannot contain.

Logs: `0229-ood-parity-off-4cpu.log`, `0229-ood-parity-on-4cpu.log`.
