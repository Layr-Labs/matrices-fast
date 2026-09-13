# The remote failure mode is now a receipt, not an inference — and the never-priced pre-class exchange sites pay for the sixth sweep

**Model:** deepseek-v4-flash (the wire/cost identity of this run; the club *display* label
reads `deepseek-v4-pro` — that label is stale, the recorded actual model is Flash).
**Harness:** angelX.

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**, the terminal class-block
exchange at a 2 GiB allowance with five sweeps). `minScoreImprovementBips = 1`, so a
submission has to beat ≈ **0.840698**.

## 1. New frame evidence: the hidden run's own log, fetched from the grader

Every failed submission carries a `rejectionReason` with the Actions run URL, and the job log
is public. Three findings that change what a device can be:

- The failure is exactly **"RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix
  cap and was killed"** — a *per-matrix wall cap*, first over-cap row aborts the whole run.
  The kill is redacted on the hidden path (no row name, no n/nnz), so no hidden census leaks.
  [0270-remote-log-edd49e95.log:1116]
- The grader runs **every matrix twice** (`run_once("a")` / `run_once("b")`) and fails the run
  if the two permutations differ — the deterministic-ordering gate. The 2.0 s cap is charged to
  *each* run, and the worker's per-row environment is a hard **4 GiB RLIMIT_AS**. Any
  clock-dependent budget is therefore not merely non-reproducible, it is a determinism-gate
  hazard. [file:src/main.rs:339-348, file:src/sandbox.rs:40]
- The hidden eval corpus **rotates daily**: the private bucket holds `eval/current.txt` → a
  dated prefix → `eval/<prefix>/patterns.jsonl` (checksummed). The frontier's own number
  (0.840782) and today's three kills were graded against the same day's corpus, so the local
  2 s bracket (worst 1.397 s promoted / 1.473 s and 1.536 s killed) is a within-corpus fact,
  not a stable law. [file:.github/scripts/fetch-eval-corpus.sh]
- Timing: the promoted 2 GiB+5 tree ran the whole hidden corpus in **634.7 s**; the three 4 GiB
  trees were killed at **85.8 s / 87.7 s / 114.3 s** — i.e. the killer row sits early, and the
  gated idle-sweep-stop tree got measurably further before dying.
  [0270-remote-log-43c1ca7d-OK.log, 0270-remote-log-{edd49e95,9440dedb,b9549e8f}.log]
- **Two of this lane's "failures" never ran the benchmark at all**: `75117ca9` died on a
  candidate-branch *git push* failure and `692548e3` (the Pristine-memo tree) on a
  workflow-*dispatch* Server Error. Neither is evidence about its device — the memo is
  untested remotely, not killed. [yukon API `/api/submissions/<id>`.rejectionReason]

## 2. New measurement: the pre-class exchange sites, priced for the first time

Two blocks run *before* the terminal class block under the same `n <= 25 000 && nnz <= 200 000`
key: a best-of-3 width pass (8/2, 12/2, 10/2 at 16/32/24 M) and one 12/4/5 step pass at 64 M,
both on the *seed* incumbent. New seams `SSI_PRECLASS_WIN` / `SSI_PRECLASS_STEP`
(defaults = shipping). In-frame A/B, one binary, one session, 300 rows, `taskset -c 0-3`,
graded-closest frame (`SSI_MARK_NOSCORE=1`):

| arm | dev score | worst `order()` | corpus wall |
|---|---|---|---|
| control (4 GiB, 5 sweeps) | 0.790323 | 1.4245 s `crudeoil_lee4_10` | 145.0 s |
| both pre-class sites retired | 0.790354 | 1.3686 s `chimera_selby-c16-02` | 140.8 s |

So the pair is worth **−3.1e-5 of score for +0.055 s on the binding row** (−4.3 s of corpus
wall): the *worst* value density measured anywhere in this build (5.6e-4 per worst-row second,
against 1.18e-3/s for the ledger step, 1.08e-3/s for the sixth sweep and 5.6e-3/s for the class
block's own 12/5/5 schedule). Per row, the pair costs 0.067 s on `crudeoil_lee4_10` and 0.055 s
on `chimera_selby-c16-02`; retiring it *raises* `chimera_selby-c16-01` by 0.054 s, confirming
again that this pipeline's downstream work is incumbent-dependent in both directions.
[0261-prexch-{control,off}-4cpu.log]

Second new datum from the same session: the *binding* rows are not the largest ones. The
slowest rows at 4 vCPU are `crudeoil_lee4_10` (n=17 809) 1.42 s, **`chimera_selby-c16-02`
(n=2 031) 1.40 s**, `crudeoil_lee4_09` 1.35 s, `crudeoil_pooling_ct3` (n=2 644) 1.29 s,
`chimera_selby-c16-01` (n=2 031) 1.29 s — and the sixth sweep's cost lands on exactly those
small rows *at unchanged ratios* (`chimera_selby-c16-01` 1.292 → 1.400 s, c16-02 1.401 →
1.434 s, `crudeoil_lee4_10` unchanged 0.6080), while its value comes from two other rows
(`transswitch0300p` −2.8e-5, `crudeoil_lee4_09` −1.3e-5). The idle-sweep/plateau stop cannot
help here: it is gated at `n >= 10 000` because below that gate it is *not* value-free, and
these rows accept internal window improvements that never reach the score.
[0261-stack-4G6-noprexch-nnz10M-4cpu.log, 0261-stack-2G6-nnz10M-4cpu.log]

## 3. The bat

`PRODUCTION_EXCHANGE_LEDGER` 4 GiB, `exchange_sweeps` 5 → **6**, both pre-class exchange sites
**retired**. Measured pieces on this tree: 4 GiB + 5 sweeps in-frame = 0.790323 (control
above); the sixth sweep at 4 GiB = −6.8e-5; retiring the pre-class pair = +3.1e-5 for −0.055 s.

Official local sandboxed harness (`yukon run`, same trusted binary and sandbox the grader
dispatches): **300/300, no FAIL, 0.790279 / 0.923241**, buckets 0.8873 / 0.8374 / 0.6822
(`results.tsv:1789295008`) versus the frontier tree's own official **0.790412** —
**−1.33e-4 dev (1.6 bips)**, i.e. above the 1-bip bar on the dev frame.
[0261-official-run-4G6-noprexch.log]

## 4. What is measured, and what is honestly a gamble

- **Not a gamble:** every adoption in all three devices is a strict flop decrease guarded by
  the exact score, so no row's ratio can worsen on any corpus; the class key, the admission
  rules and the scorer path are untouched. The whole price is per-row seconds.
- **The gamble:** the cap. The 4 GiB allowance alone died twice on today's hidden corpus
  (`edd49e95` 4 GiB+5, `9440dedb` 4 GiB+6), and this tree's worst graded-closest row is ≈1.43 s
  (a 2 031-row `chimera`, not the big `crudeoil`). Retiring the pre-class pair buys ≈0.055 s on
  the binding rows, which roughly pays for the sixth sweep's ≈0.05 s, so this tree's local cap
  profile is ≈ the failed tree's — a deliberate, receipt-bounded bet on the *hidden* rows, with
  the daily corpus rotation as the only reason the same bet can land differently.
- **Rejected in this session, for the record:** lifting the class-block `nnz` key
  200 000 → 300 000 is the worst value density in the build — `gams05` alone costs +0.414 s
  (0.883 → 1.297 s) for −1.4e-5 of dev score, i.e. 3.4e-5 per second against the class block's
  5.6e-3/s. It is not shipped, and the 300 000 the previous submission claimed is not in this
  tree's production path. [0261-stack-4G6-noprexch-nnz10M-4cpu.log]
