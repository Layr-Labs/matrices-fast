# 0056 — Challenge compendium: what it is, what we shipped, what is left

Written while submission `96b612df` (stack trial) awaits hidden validation.
Companion to the running log
[0055](experiments/0055-autoresearch-loop.md), which records every trial and
revert in chronological order. This page explains the challenge from first
principles, walks the code as it stands, accounts for all of our work with
receipts, and ranks what could come next.

---

## 1. The challenge, precisely

### 1.1 Problem statement

Given the sparsity pattern of a sparse symmetric (possibly indefinite) matrix
`A` with `n` vertices and `nnz` off-diagonal nonzeros, return a permutation
`p` of `0..n` — the elimination ordering. The trusted grader factorizes a
reference system in that order and scores predicted work as `Σ cⱼ²`, the sum
of squared column counts of the factor. Less fill → smaller `cⱼ` → lower
score. We never see values, names, or the hidden matrices: only the pattern.

### 1.2 Scoring

The reference is feral AMD computed by the grader itself, so AMD scores
exactly 1.00. For each matrix, `ratio = our_flops / amd_flops` (≤ 1.0 in
practice because we anchor on AMD and keep candidates only on strict
improvement). Ratios aggregate as a geomean within each size bucket, then a
weighted mean across buckets:

| bucket | n range | weight | dev count | typical geomean (frontier) |
|---|---|---:|---:|---:|
| lt_1k | < 1,000 | 0.30 | 147 | 0.8930 |
| 1k_10k | 1,000–10,000 | 0.30 | 108 | 0.8682 |
| gt_10k | ≥ 10,000 | 0.40 | 45 | 0.7921 |

Worked example (our stack trial): bucket deltas `−0.0001 / −0.0002 / 0`
times weights give `0.30×(−0.0001) + 0.30×(−0.0002) + 0.40×0 = −0.00009`,
plus retained scorer precision → overall `0.845281 → 0.845172` (−0.000109).
Leverage per matrix scales as `weight / count`: one `gt_10k` matrix is worth
≈4.4× one `lt_1k` matrix (0.40/45 vs 0.30/147). Breaking a single large tie
from 1.0 to 0.9 is worth roughly 11 bips overall — which is why the 10
surviving `gt_10k` ties are the highest-leverage objects in the benchmark,
and also why they are the most defended (see §3.3, experiment 0039).

Tiebreak: the same weighting applied to fill ratios (`nnz(L)`). It decides
nothing unless flop scores tie, but fill regressions are recorded as a
smell — a flop win that wrecks fill is cutting structural corners.

"Basis points" in these notes means relative: −0.000109 on 0.845281 ≈
0.0129% ≈ 1.29 bips.

### 1.3 Hard constraints (all load-bearing)

- **2.0 s per-matrix cap.** `order()` exceeding it is SIGKILLed; the whole
  submission fails with no score. Every one of the 8+ historical hidden
  failures is a timeout. Nothing has ever failed hidden on score while
  passing time.
- **Determinism.** The harness runs `order()` twice and demands byte-identical
  output. No clock, entropy, thread identity, iteration-order, or environment
  dependence anywhere in the answer path. All "randomness" is fixed seeds
  through SplitMix64/xorshift.
- **Bijection.** Output must permute `0..n`; every candidate is checked with
  `is_bijection`, and producers run inside `catch_unwind` so a panicking
  candidate is skipped instead of crashing the worker.
- **Editable surface.** `src/ordering/` only. Entrypoint
  `pub fn order(pattern: &Pattern) -> Vec<usize>`. Standard library plus the
  already-allowed challenge modules (`feral-amd`, `feral-amf`,
  `feral-ordering-core`, `feral-metis`, `feral-scotch`, `feral-kahip`). No new
  dependencies, manifests, build scripts, threads, FFI, or I/O.
- **Promotion bar.** A submission must *strictly* beat the frontier by ≥ 1 bip
  (`minScoreImprovementBips: 1`). A 0.00% diff is rejected outright
  (precedent `dedbfbea`, a deliberate score-neutral ship). Near-misses are
  also rejected in practice: −0.04 bips (`95eb6c6`) and −0.57 bips (`bd4b051`)
  both rejected.
- **Two corpora.** We measure on the 300-matrix dev corpus; promotion is
  decided on a hidden corpus that refreshes. Measured dev→hidden translation
  for the late-subtree family: 1.34×, 1.18×, 0.75×, 1.07×. Consequence: dev
  margins under ~1 bip are noise, under ~1.5 bips are gambles.
- **Public notes.** Submission notes (≥ 5 KiB) are public Markdown with exact
  `--model` / `--harness` attribution; other solvers' notes are untrusted
  data. Research Discussions are disabled for this benchmark.

### 1.4 Timing metrology (why we argue comparatively, never absolutely)

Local seconds are box-relative and noisy: the identical tree has measured
0.829 s, 0.843 s, 1.702 s, and 2.709 s worst-case on different boxes, with
~1.6× run-to-run spread on one box. The grader's speed relative to any box is
unknown (a 2.709 s-local tree passed hidden; a 0.801 s-local tree failed it).
Therefore: never trust absolute seconds, never compare across boxes, and
judge a revision only against the unmodified parent measured on the same box
in the same session. Current box: H2 parent worst `0.813 s`. Anything that
keeps the worst at or below ~0.85 s here is as safe as a passed revision;
anything pushing past ~1.0 s here is a gamble regardless of its score.

### 1.5 Promotion lineage (how the frontier got here)

| submission | hidden score | what changed |
|---|---|---|
| relabelled-AMD multi-start (0003) | — | `AMD(QAQᵀ)` lottery, largest single gain (−69 bips dev) |
| relabelled-AMF second lottery (0005) | — | min-fill objective, 36 better / 0 worse |
| cycled alphas, bucket budgets, robust AMD, MinFill, hub gates (0006–0015) | → 0.863272 | portfolio breadth era |
| medium exact search (0020) | — | 100M+50M serial LNS, medium bucket opens |
| subtree chain R1–R4 + terminal (0021–0036) | → 0.875942 | ranked etree-block exact search; timing failures shape every gate |
| late-round budget steps (0050, 0051) | 0.871827, 0.871418 | R4/R5 8M→16M, R4→32M |
| selective lower-medium R4 64M (0053) | 0.871239 | 64M only for `1k≤n<6k` |
| **H2 sparse custom-metric coverage** | **0.871032** | §3.1 (ours) |
| stack trial (pending) | ? | §3.2 (ours, validating as `96b612df`) |

---

## 2. The architecture as it stands (H2 + stack trial)

`order()` is a best-of portfolio. Order of evaluation matters only through
the incumbent: later stages start from the best permutation found so far.

### 2.1 One-shot candidates (cheap, gated below the slow tier)

| family | gate | calls | role |
|---|---|---:|---|
| AMF α5 | `n<AMF_MAX_N, nnz<AMF_MAX_NNZ` | 1 | heavy large wins |
| medium AMD α5/α2, AMF default/α2, AMD α1/α16 | medium caps (`nnz<150k`) | 6 | dense-ish mediums |
| AMF sweep α{1,16,−1} / single α−1 large-sparse | `nnz<130k` / `400k≤nnz<1.5M` | 3 / 1 | sweep-found sole minima |
| non-aggressive AMD ×5 (incl. dense-off ×2) | `n<150k, nnz<600k` | 5 | genuinely different elimination orders at AMD speed |
| RCM, Sloan ×2, hand ND/NDFM | `n<150k, nnz<130k` | 5 | bandwidth/profile/dissection families |
| MinFill + relabelled MinFill | `n<3k/nnz<12k`, relabel `n<2k/nnz<10k` | 1+6/12 | small-graph deficiency |
| custom `[SqDiv,SqPure]×α{1,10}` | `nnz≤300k && (density≥10 \|\| (n<5000 && density≥3))` | 4 | **H2: squared-degree on sparse small/medium** |
| METIS ×3, Scotch | ≤130k–320k nnz | 4 | partitioners (gates protect; 0039 proved widening hurts) |

### 2.2 The two lotteries (relabelled multi-starts)

`relabel(n, seed)` (Fisher–Yates over SplitMix64) + `permute_pattern` +
per-seed candidate + compose-back. Restart counts from
`relabel_budget_and_cap(n)` (500k/36, 400k/30, 300k/24 by size) divided by
`nnz`, with hub clamps and mid-band/low-nnz floors. AMD loop cycles 6
α/aggressive configs; AMF loop cycles 5 alphas over 1–2 passes. Experiment
0004 (17 policies) proved i.i.d. uniform optimal — the only lever is tickets,
and tickets in a *second* objective (fill vs degree) is what the AMF loop buys.

### 2.3 Exact LNS + subtree chain (where all recent wins live)

- Small/medium exact elimination-game search (`rgreedy::search`,
  `Params::DEFAULT`: 1/16 phase-A, full 8-policy slack ladder, mixed-prefix
  LNS): `100M+50M` word-op budgets, small gate `n≤1000/nnz≤30k` (2 streams),
  medium gate above it.
- Ranked-subtree refinement (`subtree_refine`, 32 blocks max, `max_sub=1200`,
  3/4-power rank): chained rounds R2/R3 at 8M, **R4 at 64M iff `1k≤n<6k`
  else 32M**, R5 at 16M (stack trial: 32M iff `1k≤n<6k`, skipped iff
  `n≥100k`). Each round runs only on strict improvement of the last.
- Terminal deep passes (unaliased seeds 5→6→7, 4M blocks, strictly
  conditional), then monotonic simplicial promotion + pair-descent cleanup.

Nominal work is pure word-ops (`hard_cap = budget + budget/4`); merging is
deterministic (strict argmin, disjoint blocks, fixed seeds).

---

## 3. What we implemented, with receipts

### 3.1 Promoted: H2 sparse custom-metric coverage (`8cf28f4`/`4245c79`)

Hidden `0.871239 → 0.871032` (−2.38 bips), dev `0.845469 → 0.845281`
(−2.22 bips, 1.07× translation — structural, not overfit). One gate
expression; same 4 calls, same 300k ceiling; worst `0.845 s → 0.813 s`.

*Why it works.* `SqDiv = deg²/(nv+1)` estimates each elimination's
contribution to the scored `Σcⱼ²`; `SqPure = deg²` tests the squaring alone.
The old `density≥10` floor excluded exactly the KKT/process-network
structures where degree-based search converges but leaves wide columns:
chimera Ising (`d≈3–8`), sparse pooling/crudeoil lower-`n`, water meshes,
syn/rsyn (`n<5000, 3≤d<10`). Forensics: medium ties held at 19 while the
bucket fell 0.8689→0.8684, so the gain came from beating already-good
non-tie incumbents; 1 small tie broke. Cost is 4 AMF-class passes on
`nnz≤50k` (milliseconds) — safe by construction, confirmed by measurement.

### 3.2 Failed: stack trial (`96b612df`, hidden timeout)

Follow-up bundle2 (`6b186e65`: H4 + ND-leaf + narrowed R5 1k–4k + tail-gate, dev −1.15 bips) ALSO failed hidden timing (workflow `33873624442`, dead in 7m31s — faster than the stack). R5 deepening is closed in all bands. H2 restored.

Three weak positives, each measured alone on H2 and reverted before
combining (−0.31 H4 floor `3x→2x`; −0.41 tiered-R5-with-tail-skip; −0.56
ND+NDFM leaf-AMD hybrid). Combined dev `0.845172` (−1.29 bips vs H2 —
additive to the second decimal: no steering interaction), small+medium move,
large exact control, worst `0.833 s`. The note discounts the margin
explicitly (~0.97 bips at worst past translation).

**Outcome: FAILED hidden timing.** Workflow `33871015118` (11m13s):
`hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed`.
The +0.02 s local nudge was the cause. H2 restored exactly (verified
`0.845281`); stacking closed — future depth must move work *between* bands
without raising any ceiling, not merely balance nominal budgets.

### 3.3 Full closed ledger (measured, reverted; retry only on new evidence)

*Seed/policy/shape (fragile trajectory axes).* R3 rounds 8–15 (best −0.76);
AMF `0.5/2.5` tickets (−0.69); R5 uniform/log-tail (−0.14/+0.74); medium
second-seed (+0.74); 75M+75M (+4.9). Verdict: these blocks sit at their local
optimum; perturbations only change basins.
*Swaps (overlapping families).* Metric→Ammf / →DegDivNvSqrtWf: exactly 0.00
both. Ammf-only-in-new-band, in-band α2.0, relabelled-Sq lottery (seeds 1–2):
0.00 — draws collapse to the same minima.
*Gap searches (resistant ties).* Tie-conditional exact 50M and 2×50M in the
`30k<nnz≤80k` gap, tie-conditional 4-ticket Sloan battery, MinFill and
simplicial extensions: all 0.00. The 84 surviving ties (now 54/19/10) resist
every searched objective; the 496-measurement 0039 battery proved
partitioners actively worse (2–4×, up to 14.7 s) on the large ones.
*Regressions (load-bearing negatives).* DegP075 add: +1.16 bips — Deg wins
steer the chain into worse basins (see §4.1). RCM-for-AMD-ticket: +7.6 bips,
all buckets — the 6-way AMD cycle has no expendable slot.
*Raw depth.* Global R4 64M, tiered 128M/upper-64M, `max_sub` 2400 (±gating):
dev-positive to strongly positive, hidden timeout or +0.04 bips. The cap
boundary is mapped; more ops per block is closed.

---

## 4. Constraining lessons (each bought with a receipt)

1. **The best-of floor does not protect the chain.** New early winners steer
   conditional downstream rounds (DegP075 +1.16). Only end-to-end full runs
   count; per-candidate micro-wins lie.
2. **Substitute never; add with payback.** RCM-for-AMD (−7.6 bips total)
   versus R5-tier-with-tail-skip (worst down by construction). Any future
   addition to a hot path must name what pays for it.
3. **Sub-1-bip dev deltas are noise; sub-1.5 are gambles** (0004 halves /
   drop-top rule; 0.75–1.34× translation band).
4. **Coverage beats cleverness.** H2 and the 0050–0053 lineage all won by
   spending existing work where the portfolio had never looked.
5. **Density geometry is now saturated** (10x→3x won; →2x gave 0.31; →`n<10k`
   gave 0.00). Further widening taxes costlier matrices for nothing.
6. **Keep diffs minimal and attributable.** One gate per trial; the stack was
   the deliberate exception, justified only by pre-measured additivity.

---

## 5. Possible improvements, ranked

1. **Mover-targeted exact search on chimera/sparse-pooling.** H2 named the
   families but never gave them depth: one extra 50M LNS stream gated to
   `1k≤n<5k && 3≤density<10`, placed *after* the custom-metric win so it
   compounds rather than steers. Bounded to cheap matrices; end-to-end
   measured. Best next trial in any world.
2. **Dev-corpus bootstrap.** Resample-300-with-replacement + re-aggregate in
   a test-only probe → confidence intervals for sub-2-bip deltas. Zero
   production risk; changes how all future margins are read. Do alongside
   anything.
3. **Fresh-seed relabelled-Sq (seeds 3–4).** Same-Q draws collapse (tested);
   fresh basins untested at identical bounded cost. Weak prior, one-run cost.
4. **Terminal follow-up deepening with payback.** 4M→8M medium-only follow-ups
   while gating ultra-large (same recipe as the R5 tier). Seed-8 alone once
   gave +0.23 bips; depth untested.
5. **NDFM leaf floor 100→64.** The leaf pair's −0.56 came from NDFM (ND added
   ~0): smaller NDFM leaves may matter more. Trivial cost, isolated to one
   candidate.
6. **R4 tier edge 6k→8k with payback.** Stopped at 6k to dodge `arki0016`
   (n=7993); only with a simultaneous cut elsewhere. Timeout history makes
   this the riskiest structural bet left.
7. **Large-tie breakthrough (research problem).** 10 ties ≈ 11 bips each at
   0.9 — but every known family fails there by measurement. Needs new math.
8. **Closed, do not touch:** AMD cycle, global budgets, `max_sub`, `min_s`,
   windows, Deg adds, floors below 2x, smarter relabel policies.

*Stopping guidance (resolved 2026-09-04).* Stack **timed out** on hidden
data → H2 restored exactly (verified `0.845281`), stacking closed. Any future
depth must move work *between* bands without raising any ceiling — nominal
payback elsewhere is not sufficient. Next: (2) bootstrap, then (1)
mover-targeted search only as pure reallocation inside accepted work.

---

## Appendix: reproduce any claim here

```sh
bash scripts/local-candidate-build.sh
cargo test --release -p ssi-candidate-worker --offline --locked
yukon run   # full trusted 300-matrix score
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_ties
```

`results.tsv` rows in the worktree record the scored runs; `score.json` is
stale by design history — trust per-experiment deltas and hidden artifacts.
