# 0148 — ML/RL-guided ordering under purity constraints (literature survey, paper only)

Question (from `open-questions.md`): do any ML/RL-guided ordering ideas
fit a stdlib-only, deterministic, 2 s/matrix `order()`? Surveyed
2026-09-10. No code, no builds, no probes — the literature plus our own
measurements close it.

## Purity constraints (non-negotiable, restated once)

stdlib-only (cargo-deny allowlist — no torch/ONNX/ndarray-class
runtimes, effectively no NN inference dependency), deterministic
byte-identical double runs, 2 s/matrix hard hidden cap, `src/ordering/`
only, structural `(n, nnz)` gates only. Any proposal must clear ALL
five; clearing four is a no.

## What the literature actually contains

**1. Learned separator policy (RL + GNN, multilevel).**
Gatti, Hu, Smidt, Ng, Ghysels, "Graph Partitioning and Sparse Matrix
Ordering using Reinforcement Learning and Graph Neural Networks",
JMLR 23(303), 2022. A2C agent + SAGE graph convolutions, recursive
coarsening with learned refinement; vertex separators feed a nested
dissection ordering evaluated in SuperLU on SuiteSparse graphs.
Headline result, quoted: "similar partitioning quality as METIS,
SCOTCH and spectral partitioning" — PARITY, not better. Small graphs
fall back to METIS outright. Dead here four independent ways: (a)
parity with METIS buys zero crown slots — METIS itself loses those
slots on our corpus (hand-ND 2.2–39726x off AMD on big ties; supply
probe best case 1.89x; the bar is beating a crown that already
contains METIS plus refiners, not matching METIS); (b) inference is
recursive coarsening plus NN refinement passes per matrix — big-row
additive work that dies on the 2 s cap by construction (our catch-22);
(c) torch-class dependency banned; (d) float NN inference across
heterogeneous boxes endangers byte-identical double runs.

**2. Learned algorithm selection (portfolio picker).**
GNN predicts resulting fill per traditional method and picks the best
(Booth-line selector, via the 2025 UDNO review, which notes it
"selects from existing methods but struggles to produce new
orderings"). Dead here: the arms (METIS/Scotch/KaHIP-class external
partitioners) are banned dependencies; per-row best-of is already
harvested downstream (refiners cover every tie-basin — md_deficiency's
63 weak-row wins vs AMD changed nothing vs the pipeline); and a learned
selector over our LEGAL arms is just hand-coded structural gating,
which we already have (`(n, nnz)` gates throughout the pipeline).

**3. Sequential elimination policy (RL/MCTS at inference).**
"Alpha Elimination" (DRL + Monte Carlo tree search + CNN Q-values for
elimination sequences); Dasgupta & Kumar 2023 (CNN policy picking
rows/columns to eliminate). Dead on arrival: per-matrix search at
inference time is the most expensive possible form — MCTS rollouts
cannot fit a 2 s budget at 100k+ rows (demonstrated at small scale
only), and MCTS nondeterminism collides head-on with the
byte-identical requirement.

**4. One-shot learned scoring (GNN node scores → sort).**
PFM / "Factorization-in-Loop" (AAAI 2025: graph encoder predicts
scores, differentiable reordering layer, l1-of-Cholesky-factor loss);
CFP self-supervised (multigrid GNN + triplet sampling, Fill-Path
consistent); UDNO-style spectral-embedding scorers ("ordering time
nearly linear like METIS/AMD"). This is the ONLY cost-plausible form —
one forward pass plus a sort. Dead anyway: (a) still needs a GNN
runtime (banned) plus training infrastructure; (b) reported gains are
against weak baselines on narrow distributions — no SuiteSparse-wide
flop-geomean win over tuned AMD-plus-refiners is claimed anywhere, and
PFM's own review concedes the one-shot line has "no theoretical
guarantee"; (c) it is formally a learned feature-scoring function, and
that family is saturated HERE by direct measurement (see §5).

## The measurement wall (our data, stronger than any citation)

- md_deficiency tiebreak: 63 weak-row wins vs RAW AMD incl. the biggest
  weak margins of any mechanism (−3394/−3466/−3119 row-bips) — and
  SCORE-identical vs the pipeline. Smart ties win big vs AMD and change
  nothing vs refiners. A learned scorer harvests exactly these basins,
  which are already harvested.
- AMD-only relabel lottery (`51.53`, dev −1.15): lottery, not mechanism —
  9/9 wins secondary-on slots, nothing translated. Any weights fit to
  300 dev rows face the hidden rotation every round (RULES) — the same
  wall as every lottery we closed.
- Synth corpus cannot supply substitute labels: TX2/TX2R rebuilt on
  66 synth rows with IDENTICAL cool envelopes — synthesis cannot
  discriminate the mechanisms we need discriminated.
- Translation bands 0.25x–1.4x mean a learned scorer needs 2–10x local
  margins to survive the trip; no published one-shot scorer shows
  anything near that over strong baselines.

## The only purity-fitting form, and why it is already closed

A hand-coded feature-scoring model (linear weights on structural
features, zero ML dependency) clears all five purity constraints — and
IS the saturated family: degree/deficiency/hybrid scores, weighted
lotteries, MD-deficiency, AMF-α variants all closed by measurement.
There is no evidence path from "hand-tuned scores" to "learned linear
weights" that survives rotation, and fitting needs labels unobtainable
honestly (hidden rotates; dev-fit = lottery; synth = undiscriminating).

## Verdict: CLOSE with citations

No ML/RL form fits `order()` under the five constraints; the one form
that fits the constraints (hand-coded scoring) is measured-saturated;
the forms with any literature momentum fail banned-deps, cap,
determinism, or the parity-isn't-enough bar. This stops re-litigating.

## Falsifiers (re-open conditions, any one)

1. A published one-shot scorer with microsecond/row CPU inference,
   zero-dependency implementable, trained strictly off-dev, showing a
   SuiteSparse-wide flop-geomean win over tuned AMD-plus-refiner
   baselines. None exists as of 2026-09-10.
2. Dependency policy change admitting a deterministic inference runtime
   AND a determinism solution for float inference across boxes — then
   revisit Gatti-class separators, which must still BEAT (not match)
   METIS to earn a seat.
3. Evidence the hidden corpus stops rotating — then dev-fit weights
   become viable and the training-data wall falls. Check rotation first
   (the board itself is the free clock: promotions = rotation signal).
