# 0151 — Local cap forensics, and the two independent-set arms are value-equivalent

Date: 2026-09-11.  Base: `ab30c0e` (promoted frontier `a9905f2`, remote 0.842857).
Instrument: the sandboxed `probe` module (see "How to measure" below). No candidate
was kept; this page is a negative result plus two measurements the harness hides.

## 1. The 2 s cap FAILs on this box are environmental, not algorithmic

The frozen frontier cannot finish a full local `cargo run --release` here. **Three**
consecutive runs of the *unchanged* worker each died on a **different** matrix:

| run | row reported by the harness | n | nnz(A) | result |
|---|---|---:|---:|---|
| `results.tsv` row 1 | `pooling_sppc1pq` | 14100 | 477680 | FAIL, "exceeded the 2.0s cap" |
| `results.tsv` row 2 | `p_ball_10b_7p_3d_h` | 1236 | 4186 | FAIL, "exceeded the 2.0s cap" |
| `results.tsv` row 4 | `sonet24v5` | 874 | 7260 | FAIL, "exceeded the 2.0s cap" |

Run 3 got ~220 rows deep before dying, runs 1 and 2 ~40; the failing row is a
different one every time and two of the three are trivially small (n <= 1236,
nnz <= 7260 — sub-millisecond `order()` work). Timing the same rows directly:

- `SSI_PROBE_ONLY=pooling_sppc1pq SSI_PROBE_REPEAT=3` → `order()` = **0.377 / 0.385 / 0.396 s**
  (phases: `1.portfolio` 0.155 s, `1b.indep` 0.082 s, `13.alt` 0.084 s; final ratio 0.1683).
- `SSI_CORPUS_FILE=<one-row corpus> cargo run --release` for `p_ball_10b_7p_3d_h`
  → the *whole* run (load + 2 sandboxed worker spawns + scoring) passes in **0.96 s wall**
  and writes `results.tsv` row 3 (`OK 0.973913`).

So neither row is near the cap in the algorithm, and the tiny row is nowhere near it
even end-to-end. The difference between the failing and passing contexts is the
full-corpus run itself: the parent holds all 300 patterns (~1 GB) and this host is a
shared 15 GB desktop box (load average ~5.8, 1 GB swap in use, desktop + editor
processes resident). Conclusion (hypothesis, not proven mechanism): the 2 s wall-clock
cap is being consumed by host contention / page-cache and swap effects around the
sandboxed worker spawn, not by `order()`.

**Consequence for the loop:** a full local `cargo run` is not a reliable verifier on
this host. The in-process probe is. Every number on this page after this section was
taken with the probe.

## 2. Full-corpus baseline on the frozen frontier (probe)

`probe_timing_and_score`, 300 rows, one pass, 113.7 s:

```
lt_1k    count=147   geomean=0.8875
1k_10k   count=108   geomean=0.8397
gt_10k   count=45    geomean=0.6857
SCORE = 0.792439
WORST order() = 1.085 s      (crudeoil_lee4_09, 1.0853 s)
```

This reproduces the last recorded dev number (0.792439, 0149/0150) exactly, so the
checkout and the instrument both agree with the inherited ledger. Ties
(`ratio >= 0.9999`): **77 / 300** rows. Slowest rows after `crudeoil_lee4_09`:
`crudeoil_lee4_10` 0.990, `crudeoil_lee4_06` 0.951, `gams05` 0.901, `nuclear104` 0.897.

## 3. The two independent-set arms are equivalent on all 300 dev rows

`indep_first.rs` carries two entry points: `run_general` (the open-set path: AMD on
every admitted set, then AMF / METIS / quotient metrics on the competitive cores) and
`run_sequential_180` (the 180a family). Since iter235a the window
`(1800..=2500).contains(&n)` **replaces** the first with the second — a size window that
swaps in one algorithm for every pattern of that size.

Hypothesis: the general arm has value inside the band that the substitution discards,
so a best-of-both mix should convert some band rows.

Change (reverted, not kept): `run` became a dispatcher returning the cheaper of both
arms inside the band and, outside the band, the cheaper of `run_general` and a
second `run_sequential_180` gated by the monotone pair `n <= 25_000 && nnz <= 100_000`
with a halved admission ledger.

Measured on the whole corpus (probe, same instrument as the baseline). Raw logs kept
in-repo as [`memory/evidence/0151-probe-baseline-ab30c0e.log`](../evidence/0151-probe-baseline-ab30c0e.log)
and [`memory/evidence/0151-probe-both-arms-mix.log`](../evidence/0151-probe-both-arms-mix.log)
(`SCORE` at their line 617):

| run | SCORE | improved rows | regressed rows |
|---|---:|---:|---:|
| baseline `ab30c0e` | 0.792439 | — | — |
| both-arms mix | **0.792439** | **0** | **0** |

Every one of the 300 rows produced *byte-identical* flop counts (e.g.
`maxcsp-ehi-85-297-71` 1110971341, `pooling_sppc1pq` 136686008, `catmix400` 36364
before and after); the two evidence logs differ only on their timing lines
(`WORST order()` 1.085 s vs 1.039 s, "matrices under 0.400 s" 180 vs 170). Targeted run
on the 17 rows in the band alone: identical flops on all 17, order() times unchanged
within noise (0.78 s `chimera_selby-c16-01`, 0.74 s `-c16-02`, 0.34 s `squfl025-030`,
0.29 s `knp5-44`).

Reading the two facts together:

- outside the band, `min(general, 180) == general` ⇒ **the 180a arm is never strictly
  better than the general arm outside `1800..=2500`**;
- inside the band, `min(general, 180) == 180 == baseline` ⇒ **the general arm is never
  strictly better inside it**.

So on the dev corpus the two arms are value-equivalent, the substitution window is
score-neutral, and running both buys exactly nothing while costing a second bounded
lift. The experiment was therefore reverted, and nothing was submitted (a 0-bip change
cannot clear `minScoreImprovementBips = 1`).

## 4. Acceptance arithmetic (why micro-tunings never land)

`yukon benchmark show` reports `current best 0.842857`, `minScoreImprovementBips: 1`,
`claimed score: recorded only` (no `--claimed-score` needed). One bip on the remote
scale is ~0.0000843 absolute, i.e. ~0.01 %. Recent near misses that were *better* and
still rejected: `6664fb0` 0.842835, `0dd301d` 0.842833, `8156a16` 0.842833 — all
0.24–0.29 bip, a factor ~4 short. Local dev gains need to be several bips to have a
chance of clearing the bar; the only recent change that cleared it comfortably was a
structural addition (7624ac1, hidden −0.44 %).

## How to measure (kept for the next session)

`target/` is gitignored, so these helpers can never enter a submission archive:

- `target/probe-sandbox.sh build` / `run` — compiles and runs the candidate worker's
  `#[cfg(test)]` probe module **inside the same bubblewrap boundary** the harness uses
  for candidate builds (`scripts/grader-sandbox.sh candidate-build`), with a persistent
  target dir (`target/probe`) so a rebuild costs ~15 s. Never bare-build the worker.
- `bash target/probe-sandbox.sh run` → full corpus, prints `SCORE`, `WORST order()`,
  the per-row table and the per-bucket geomeans (113 s).
- `SSI_PROBE_ONLY=a,b,c SSI_PROBE_PHASES=1 SSI_PROBE_REPEAT=3 bash target/probe-sandbox.sh run`
  → phase-level timing for named rows (`SSI_PROBE_ONLY` must be non-empty).
- `target/probe-diff.py <baseline.log> <candidate.log>` → exact per-row flop deltas,
  improved/regressed counts and both scores.
- One-row corpora for harness-level reproduction: extract with
  `rg '"source": ?"<name>"' corpus/dev/patterns.jsonl > /tmp/one.jsonl` and run
  `SSI_CORPUS_FILE=/tmp/one.jsonl cargo run --release`.

## Next steps this page leaves open

- The environmental-stall explanation for local cap FAILs should be re-tested when the
  host is quiet; if it holds, past local timing deaths (fe871f1, 176a/173a) are partly
  host artifacts and the real remote margin is larger than the ledger assumes.
- The 77 ties remain the whole headroom. Nothing in the two-arm experiment touches
  them; a candidate that converts ties must be a *different* mechanism, not another
  arm of the same lift.
