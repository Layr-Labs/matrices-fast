# 0292 — iter88: the graded frame's own contract, and a single-row flip in the graded score

State: **no candidate shipped this iteration** (only the shipped tree was re-measured). The
in-flight bat `f2176d38-21d3-4c40-9834-024abfd7bba7` (bat `8c3e7051`) is `validating`.
The one device built this iteration was measured in the graded frame and **reverted** (see §3).
`git status` for `src/` is clean: the working tree is exactly the committed tree.

## 1. The graded contract, read out of the grader's own files (new)

* **`order()` is run TWICE per matrix, in two separate child processes, each independently
  capped at 2 s**, and the two permutations must be equal or the matrix FAILs
  (`run_once("a")` / `run_once("b")`, `perm1 != perm2` → "nondeterministic ordering").
  So per matrix the cap is two draws, and each call's process + sandbox startup is *inside*
  the 2 s `[src/main.rs:279,332,344,311,369]`.
* The cap is **wall-clock** (`Instant::now()` … `start.elapsed() > cfg.time_cap`, poll 10 ms)
  `[src/watchdog.rs:19,111]`; the kill is a process-group SIGKILL.
* Worker rlimits are fixed and generous: `RLIMIT_AS = 4 GiB`, `RLIMIT_NPROC = 4096`, output
  `RLIMIT_FSIZE = 8 + 8n + 4096` `[src/sandbox.rs:40,49,82]`.
* The harness **never prints a per-row wall**: the census's time column is the literal
  `(capped)`. Any per-row timing claim about a graded run is an inference `[src/main.rs:417]`.
* The graded corpus is fetched by the workflow from a **dated, rotating prefix** and
  sha256-verified (`eval/current.txt` → `eval/<prefix>/patterns.jsonl`); the killed bat's log
  shows `EVAL_BUCKET_NAME: ***` (set) plus `fetch-eval-corpus: pointer resolved` /
  `checksum OK` / `corpus downloaded and verified`
  `[.github/workflows/benchmark.yml:143] [.github/scripts/fetch-eval-corpus.sh:7]`
  `[.scratch/iter87/remote-4c4f7b71.log:1129]`. So the graded set is *not* dev, and the
  platform's own ledger is the only place scores are comparable day to day.

## 2. The engine's thread budget vs the grader's vCPU budget (priced)

`PAR_MAX_THREADS = 4` is justified in-source by "the grader is a 4-vCPU runner" and by
"local timing on a bigger box must not model a different grader"
`[src/ordering/parallel.rs:47,60]`. The **one** production site that exceeds that budget is the
basin fork: two concurrent full pipelines `[src/ordering/mod.rs:1502]`.

Fork charge, same 30-row corpus (heavy set ∪ fork band, `.scratch/iter84/fd-A.tsv` names),
matched control, probe frame:

| frame | fork ON | fork OFF | charge |
|---|---|---|---|
| `taskset -c 0` (iter84, old tree) | 43.00 s | 37.64 s | **+5.36 s (+0.179 s/row)** |
| `taskset -c 0-3` (this iteration) | 28.63 s | 25.58 s | **+3.05 s (+0.102 s/row)** |
| unpinned 24 CPUs (this iteration) | 26.18 s | 24.01 s | **+2.19 s (+0.073 s/row)** |

Score is frame-invariant (0.695225 ON / 0.695672 OFF in both frames).
Per row at 0-3 the charge is `st_bsj2` +0.593, `ex6_1_4` +0.557, `prob02` +0.499,
`pooling_haverly1pq` +0.441, `rocket50` +0.207 … `gancns` +0.146, everything else ≤ 0.05.
`[.scratch/iter88/F1-4c.tsv, F0-4c.tsv, F1-any.tsv, F0-any.tsv]`

## 3. DEVICE KILL — the fork's tiny-class charge is a probe-frame artifact

Full dev corpus, **probe frame at cpuset 0-3**, one build, three arms (the new
`SSI_BASIN_FORK_MIN_NNZ` seam made the middle arm possible):

| arm | gate | SCORE | Σ order() wall |
|---|---|---|---|
| A2 | production (`n<=600 && nnz<=5000`) | 0.789420 | 185.96 s |
| E | `100 < nnz <= 5000` (candidate device) | 0.789420 | 166.92 s |
| D | fork off | 0.789502 | 155.96 s |

Per-row ratio join over all 300 rows: **E vs A2 = 0 differences** (the device is exactly
value-neutral); D vs A2 = 4 (the three band movers plus `netmod_kar1`, §4). The fork's entire
dev value, 8.9e-5, is three rows with `nnz > 100`: `chimera_mgw-c8-439-onc8-001` −0.0175,
`waterund14` −0.0084, `gancns` −0.0014. The 36 in-gate rows with `nnz <= 100` are worth
**exactly zero** and paid 12.36 s of probe wall (20.69 → 8.32 s; +0.45..+0.53 s each).
`[.scratch/iter88/TINY-A2.tsv, TINY-E.tsv, TINY-D.tsv]`

**But in the graded frame the charge does not exist.** A 4-row corpus of exactly those tiny
rows (fresh worker process per call, official harness), 4 reps per arm:

* fork ON for the tiny rows (old gate): 1.95 / 1.95 / 1.97 / 1.97 s
* fork OFF for them (device):        1.97 / 1.97 / 2.03 / 1.95 s
* identical score/tiebreak in all 8 runs (0.851716 / 0.942559).

Full-corpus official local sandboxed harness: **284.2 s** with the device vs **286.1 s** with
the shipped gate (and 284 s recorded for the shipped tree in iter87) — i.e. no graded-frame
wall change. The device was therefore reverted; `src/ordering/mod.rs` is byte-identical to the
committed tree, `directive cargo test --release -p ssi-candidate-worker` = **126 passed / 0
failed**. `[.scratch/iter88/one4.txt, one4-{DEV,OLD}-*.log, official-tinygate2.log, official-reverted.log]`

## 4. NEW — the graded score itself is not reproducible at the 1e-5 level: one row flips

Same committed tree, official local sandboxed harness, two runs in different sessions:

| run | `netmod_kar1` flops (n=1746, nnz=4928) | corpus score |
|---|---|---|
| `.scratch/iter87/slot-run.log` | 43 864 (ratio 0.7720) | 0.789998 |
| `.scratch/iter88/official-reverted.log` | 44 003 (ratio 0.7740) | 0.790005 |

**299 of 300 rows are bit-identical**; only `netmod_kar1` moves (+0.32 %), worth +7e-6 of
score. 20/20 single-row runs of that matrix in the graded frame read 0.774183, so the value is
stable *within* a session; the same row also flipped in the probe frame (0.7717 vs 0.7742),
i.e. the flip is row-specific and frame-independent. The harness's own two-call determinism
gate does **not** trip (both calls inside a run agree), and 300/300 rows pass with 0 FAIL.
`[.scratch/iter88/official-reverted.log, .scratch/iter87/slot-run.log, .scratch/iter88/one1.txt]`

Consequence: a fixed tree's dev score carries a ±7e-6 band (one 1k_10k row at 0.32 %); any
device price quoted at that resolution (the iter87 "pool slot +7e-6", "fork band −6e-6") is
**inside the noise**, while on a gt_10k row the same 0.32 % would be ≈2e-5 — the same order as
the 0.22 bip by which `3587d1b` missed promotion.

## 5. The in-flight bat resolved during this iteration: the 12-sweep registration profile died AGAIN

`f2176d3` (fork@600 + additive registration, 12 sweeps) → **failed**: benchmark step
`03:49:57.518 → 03:51:24.701Z` = **87.18 s**, verdict `RUN FAILED: hidden matrix: order() exceeded
the 2.0s per-matrix cap and was killed` (run 34803725234, job wall 5m13s).
That is the *seventh* same-corpus cap kill and the fourth inside the 81–87 s cluster
(81.5, 84.56, 86.0, 87.18 s) — i.e. the killer row is reached after ~85 s of graded wall.
Combined with finding 13 (fork retired + registration also died at 84.56 s) the registration dies
on the cap **with and without** the fork, while the same tree at 6 sweeps completes (`0df9f508`).
`[.scratch/iter88/remote-f2176d38.log:1146, .scratch/iter88/board.txt]`
