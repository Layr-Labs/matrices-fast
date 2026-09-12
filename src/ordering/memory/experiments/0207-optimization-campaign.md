# Further optimization campaign, September 12 UTC / September 13 IST

The user authorized continued optimization until more than three submissions
are rejected. This campaign starts after promoted submission `07f0e8a2`.
The stopping condition is four official `rejected` outcomes in this campaign;
workflow `failed` outcomes are recorded separately. Historical outcomes do not
count toward this new campaign.

Initial promoted source: `52affcba3180dbebd8693d28fd879ae09b161efe`.
Initial local source: `3995099`, with Rust identical to tested `433fa88`.
Initial hidden score: **0.842377**, fill **0.944856**.
Initial independently verified public score: **0.791864560331**.
Winning source remains preserved on `factor-bounded-terminal-polish`.

## Outcome ledger

- Rejected: **1 / 4**.
- Failed workflows: **3**.
- Promotions: **0**.
- Current best: `07f0e8a2`, hidden **0.842377**.
- Attempt 1: **`91dac999-6923-4b20-b2c1-38a7e1965d94`**, **FAILED hidden 2 s cap**;
  local submitted commit **`aff9f5ed8666c808a1cc0d914aa561691e652edf`**,
  public **0.791821**, fill **0.924224**, note [0208](0208-small-factor-post-window-refinement.md).
  Remote **`e15eaefdc7e4b1b701af61909494baf676891e27`** byte-identical to
  submitted ordering; workflow **34713361492**, Benchmark
  **19:13:34.69 -> 19:15:20.47 UTC**, **105.78 s**. No hidden score.
- Attempt 2: **`5a8c5623-8104-47e8-9833-7903495f7caa`**, **FAILED hidden 2 s cap**;
  submitted **`12e8c2b909653efcd836fe681883efbc44a6060a`**, public **0.791703**,
  fill **0.925316**, note [0209](0209-retained-independent-core-metric.md).
  Remote **`523f981fd6fb79a3c1ab985a659ce9d81c05501d`** matches the entire
  tested ordering archive. Workflow **34714573617**, Benchmark
  **19:38:20.18 -> 19:40:04.72 UTC**, **104.54 s**. No hidden score.
- Attempt 3: **`77d73135-8e57-4f94-8d59-4bee8ef04f8b`**, **FAILED hidden 2 s cap**;
  submitted **`ec939205276300542d22cd8ae56605ac8a2943a8`**, public **0.791703**,
  fill **0.925316**, note [0210](0210-sparse-deficiency-runtime-fund.md).
  Remote **`8a45a3fcbeb34208843d5d46cf7f3ffa8a9171d1`** matches all tested
  ordering. Workflow **34715439117**, Benchmark **19:56:20.58 -> 19:58:02.04 UTC**,
  **101.47 s**. No hidden score. Runtime kernels alone did not resolve this run.
- Attempt 4: **`30eb4c76-9ae7-42a2-908b-74e3787c6ae4`**, **REJECTED tied incumbent**;
  submitted **`9bc3e9452423db39ee476705bd09d1788153633b`**, public **0.791635**,
  fill **0.924192**, note [0211](0211-early-budget-and-winning-core-peo.md).
  Remote **`730dfd052df11500a3d73c1ff37806147c69314c`** matches all tested
  ordering; workflow **34716429064** SUCCESS, Benchmark **20:15:44 -> 20:24:40 UTC**,
  approximately **536 s**. Hidden **0.842377**, fill **0.944856**, same as leader.

## First hypothesis

The final sparse-span and narrow windows may change the completion seen by
the preceding PEO extraction. Measure another bounded PEO extraction from
the finished promoted incumbent, small fill-edge deletion allowances, and
alternative window spans followed by PEO. Every diagnostic replacement must
be a bijection and a strict decrease in exact original-pattern flops.

The test-only cache is rebuilt from the promoted winner, not the older
`c5e6c2ff` incumbent. It is verified against all 300 public matrices and the
bucket-weighted AMD ratio. Cached names and permutations are diagnostic data
only and are never compiled into production.

Production code is unchanged during this initial screen. The screen uses
dimension <= 12,000, pattern nonzeros <= 200,000, exact flops <= 20 billion,
and factor nonzeros <= 150,000. Subsequent completion candidates retain their
own materialization bound. No wall-clock gate, identity gate, dependency, or
trusted-harness change is introduced.

## Initial measured screens

The promoted winner was recomputed on all 300 public matrices: exact weighted
score **0.791864560331**, 89.695933 seconds of diagnostic ordering time. This
cache is separate from the older `c5e6c2ff` cache. All cached permutations are
freshly rescored against AMD in each screen.

[First-screen evidence](../evidence/0207-first-screen.tsv) includes every
matrix and aggregate. Two post-window PEO rounds reach **0.791846353697**,
three medium-bucket wins, 0.134807 seconds total extra diagnostic CPU. Six
rounds reach **0.791846157813**, only a 0.000000196 further gain. A one-million
fill-edge watcher reaches **0.791860434705**, two wins; two and four million
produce the same scores at higher cost. The combined PEO/watch/PEO arm
reaches **0.791842318516**, four wins. Alternative span 64 / stride 27 and
span 32 / stride 13 followed by PEO reach **0.791838969219** and
**0.791839542283**, seven and eight wins. The independent minimum across
all twelve arms is **0.791824965594** on fourteen matrices; this is an oracle
measurement, not an implemented production result.

[Priority-screen evidence](../evidence/0207-priority-screen.tsv) covers six
heap MCS policies, each with and without following PEO. The strongest arm,
high-original-degree priority followed by PEO, reaches **0.791841004430**,
five wins, 0.445506 extra seconds, max 0.032018 seconds. Lower-original-degree
and lower-completion-degree variants do not improve on the cheaper ordinary
post-window PEO arm once ordinary PEO follows. Column-count and reverse-position
priorities alone yield zero public gains. Heap variants remain test-only.
See [primary MCS literature](../literature/0207-mcs-structural-ties.md).

The next diagnostic screen tests distinct short fixed-seed exact greedy
searches from the completed incumbent and following bounded refinements.
The first proposed submission is documented in
[0208](0208-small-factor-post-window-refinement.md). It uses ordinary PEO at
150k factor entries, then watcher 1M, four 16M windows and final PEO under a
75k factor gate. Exact public score **0.791821436276**, official sandboxed
score **0.791821**, fill **0.924224**; all 300 actual counts match, 13/0/287.
All 123 active tests and 44 generated paired orders pass. Upload and hidden
outcome are recorded above; rejection counter remains zero until an official
rejected outcome. This candidate was uploaded at approximately 19:09 UTC.

The failed first tail is removed from production in the next branch. The
next hypothesis reuses at most two independent-set cores already built by
the promoted parent, adding one final AmindNorm metric under 18k/80k input,
150k core nnz, sparse-core density, 200k exact factor and 1.5× final-incumbent
AMD competitiveness limits. Rejections remain **0/4**, failed workflows **1**.

The bounded retained-core candidate is documented in
[0209](0209-retained-independent-core-metric.md). It preserves the original
driver's returned flops and permutations on 218 paired public inputs. One
late AmindNorm metric with an 80M deterministic pivot-width/scan allowance
reaches **0.791703147252**, one gt_10k win and zero losses. Added diagnostic
work is **0.240639 s total / 0.010593 s max**. The final 44 paired stress
orders and **124 active tests / 59 ignored** pass, full suite **80.03 s**.
Sandboxed verification passes all **300** at **0.791703**, fill **0.925316**,
with every count matching the independent screen. Fill increases while primary
flops decrease. Attempt two failed the hidden cap; no hidden score.

The next revision, [0210](0210-sparse-deficiency-runtime-fund.md), funds runtime
by sparse-word deficiency scans and checked x86 popcnt entry points that keep
all logical charges and choices unchanged. AmindNorm's allowance is scaled
to four times exact incumbent flops, clamped 1M..80M. All 300 complete public
permutations remain identical to 0209, score **0.791703147252**. Seven game
state/RNG tests pass. Other retained metrics have zero gains; wider windows
have only tiny gains and are not included. All **124 active tests / 61 ignored**
pass in **79.72 s**; 44 paired generated orders pass in **24.21 s**, maximum
new minimum **0.994682 s**. Sandboxed all-300 score **0.791703**, fill
**0.925316**, every actual count matches. Attempt three is validating.

Attempt three subsequently failed the hidden cap (101.47 s Benchmark). The
next [0211](0211-early-budget-and-winning-core-peo.md) halves high-flop producer
batches 64->32 above 20B flops, avoids doomed runner-up clones with exact stable
ledger equivalence, and adds one-round PEO only after a strict late core win.
Fresh all-300 permutations match the independently polished control, score
**0.791635371701**, one further win. Full **125 active / 64 ignored** passes;
48 direct PEO calls and 44 complete generated orders pass. An independent
64/32 comparison has **zero score losses on all 11 fixtures**, saves roughly
0.19–0.20 s on two >20B-flop generated graphs. Sandboxed all300 passes at
**0.791635**, fill **0.924192**, every actual count matches. Attempt four validating.

Attempt four completed successfully and was rejected at **0.842377**, fill
**0.944856**, tied with the current promoted leader. This establishes a valid
cap32 baseline and increments only the scored rejection counter to **1/4**.
The next [0212](0212-compact-search-on-retained-cores.md) projects the completed
permutation onto existing cores and searches at most one noncertified core,
under 100k full/core factor limits and an 8M..240M replay-derived allowance.
Compact exact public **0.791316868681**, one gt_10k win, zero losses, added
diagnostic **0.798339 s / max 0.059367**. Fresh all300 permutations match;
full 125 active / 66 ignored and 44 generated repeated orders pass. Sandboxed
all 300 completes at **0.791317**, fill **0.924124**, every exact count matching
the independent compact screen. Attempt five ready for upload; no new hidden
gain is assumed. Current promoted frontier remains 07f0e8a2 at 0.842377.
