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

- Rejected: **0 / 4**.
- Failed workflows: **0**.
- Promotions: **0**.
- Current best: `07f0e8a2`, hidden **0.842377**.

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
outcome are pending; rejection counter remains zero.
