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

- Rejected: **3 / 4**.
- Failed workflows: **10**.
- Promotions: **0**.
- Current promoted best: `fe4f40c` (newjordan), hidden **0.841858**, fill
  **0.944586**, source **256152b3da9b08028ab82c90f373c9e64429c18f**.
- Our promoted best: `07f0e8a2`, hidden **0.842377**.
- Our best raw scored candidate: **bb07f71e**, hidden **0.841858**, fill
  **0.944586**, rejected as a tie with newjordan's promoted global best. Our own
  promoted best stays07f0e8a2; previous rawde17cd31 was0.842374.
- Attempt14 prepared: [0224](0224-shared-final-ordering-budget.md),one finalbudget
  across terminal/core/original work. Root state marks attemptedwork,original
  onlywhenunused,n6..50k/input180k/factor750k/AMD1B/strictincumbentAMDwin.
  Exactpublic0.790539040043,7 wins/0 losses vsvalid12;actualsandbox0.790539/fill0.923724,
  all300 tuplesexact.127release/32mesh calls pass,5mesh wins/2second acceptances.
  Freeze beforeupload,coauthornewjordan;no14upload/hiddenresultyet.3rej/10failed.
- Attempt13: **02461af5-12a8-43a7-bba6-c60d1f2526dd**, **FAILED hidden2.0s cap**,
  tested/uploaded**52e28c0c62582960403813e202f3c26881250118**,entire remote
  **0f1f033a3f152406aa38f8ea7bd30bf1c815fc60** verified. Workflow**34725795538**,
  Benchmark shell**2026-09-12T23:38:35.9139668Z ->2026-09-12T23:40:20.7965061Z**,104.88s,explicit hidden cap error.
  Local0.790536/fill0.923722 is not hidden score. Campaign3 rejected/10 failed.
  The next candidate allocates one final-stage budget across terminal windows,
  retained-core work and original-edge search using actual attempted-work state.
  Original secondPEO4 research0.790507904401 remains test-only on failed13;
  no thirteenth validity/no deeper production activation is claimed.
- Attempt 12: **bb07f71e-4feb-4d61-ad42-4bd1e4104f83**, **REJECTED #3**,
  hidden **0.841858 / fill0.944586**, tie currentfe4f40c. Tested/uploaded
  **2068133ecf5915f52ffef8606f6d073ac199f141**, entire remote
  **0f006f4d36f270780a01284efb8632794717d220** verified. Workflow
  **34724755761 SUCCESS**, Benchmark **23:14:14 ->23:23:26 UTC**,552s by job times.
  [0220](0220-shared-terminal-core-budget.md): shared final-stage terminal/core
  budget; local0.791087/fill0.924023,127 release/64 generated calls/sandbox pass.
  Coauthornewjordan. First valid combined schedule/core source; original-edge
  joint sequence now eligible for production activation. Campaign3 rejects/9 failures.
- Attempt 11: **`2a68edc7-d876-46c1-a393-43b084ec603a`**, **FAILED hidden 2.0 s cap**;
  tested/uploaded **0edf04060e177ea6fcfc6224b2214f9833f4288c**, remote
  **8586225a85f5d7522639d51f29d6c391b1101ad2**, entire ordering tree verified.
  Local **0.791087 / fill 0.924023**, [0219](0219-scored-core-and-promoted-window-baseline.md),
  coauthor **newjordan**. Workflow **34724057192** Benchmark shell
  **22:59:31.754108 -> 23:01:17.330502 UTC**, **105.58 s**, explicit hidden cap error.
  No hidden score. Campaign **2 rejected / 9 failed / 0 own promotions**.
  Next: mutually exclude the terminal window work and late retained-core search,
  preserving the public core wins outside the terminal class. Original-incidence
  larger-pattern research is test-only: **0.790789084587**, eight strict wins,
  maximum helper **0.055402 s** on ARM; no hidden validity claimed.
- Attempt 10: **`074ed992-338a-441c-b6f1-782566f5f425`**, **FAILED hidden 2.0 s cap**;
  tested/uploaded **5b867796027e46630aad4baa14d763111bb6e0eb**, local
  **0.790177**, fill **0.923617**, [0217](0217-promoted-window-schedule-integration.md),
  coauthor **newjordan**. Wider promoted schedule and two completion cost gates;
  remote **74ff8114d805c500659bdc3c2ab20ac6eb2d57b6** matches entire tested
  ordering tree. Workflow **34723170151**, Benchmark shell **22:39:22.663440 ->
  22:41:07.525343 UTC**, **104.86 s**, explicit hidden `order()` cap failure.
  No hidden score; counters **2 rejected/8 failed**. Wide completion stages
  and medium fence are removed from next production; return to scored core
  continuation and the externally validated promoted terminal schedule.
Next [0219](0219-scored-core-and-promoted-window-baseline.md) returns to the
scored core continuation and newjordan's promoted terminal profile, with all
wide completion stages test-only and medium fence disabled. Public exact
**0.791087437358**,21 wins/2 losses/277 ties vs scored05, max0.735453s.
127 active tests and64 generated calls pass(3 wins/0 losses/13 ties vs failed10).
Required sandbox running, no eleventh upload yet; campaign2 rejected/8 failed.

Next [0217](0217-promoted-window-schedule-integration.md) integrates newjordan's
verified promoted terminal schedule with two measured completion cost gates.
Public **0.790176632480**, 22 wins/1 loss/277 ties vs attempt9. Only run
ordinary PEO after strict priority win andLexBFS after priority helper returns
a win; both gates preserve all 300 exact public permutations. Finished-completion
kernel old/new outputs equal, PEO reconstructions 302->16, LexBFS calls 290->4,
total 1.144744->0.617229 s ARM. Fresh no-override production check running;
original-incidence prototype remains cfg(test) only. No tenth upload or hidden grade yet.

- Attempt 9: **`2ee07107-6f3b-4e49-ad36-9abb2d6af53e`**, **FAILED hidden 2.0 s cap**;
  tested/uploaded **b13227d23af17d2a6d3beb0c59292b4d58035a1e**, local
  **0.790415**, fill **0.923716**, [0216](0216-medium-cost-candidate-batch-fence.md).
  Medium-cost producer prefix100M/64 retains20B/32. Entire remote
  **11eb4afb65b1c570e005265db1534d91e2a1f24a** ordering tree matches tested source.
  Workflow **34722194684**, Benchmark shell **22:18:04.084751 ->
  22:19:47.045357 UTC**, **102.96 s**. Explicit hidden `order()` cap failure,
  no hidden score or identity. Counters **2 rejected/7 failed**; external
  promoted frontier remainsfe4f40c at0.841858.
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
- Attempt 5: **`de17cd31-83e4-4d84-9e30-a05d65efc72a`**, **REJECTED below promotion floor**;
  submitted **`bf72aacdfabe3985226e13d429e63a1082def316`**, public **0.791317**,
  fill **0.924124**, note [0212](0212-compact-search-on-retained-cores.md).
  Remote **`f9c69b0a7181b37d32e2e834ffb714c1ab1f817d`** matches all tested
  ordering; workflow **34717388778 SUCCESS**, Benchmark **20:35:02 -> 20:44:03 UTC**,
  **541 s**. Hidden **0.842374**, fill **0.944855**, displayed primary gain
  **0.000003**, approximately **0.0356 relative basis points**. Valid raw
  improvement, not promoted. Counters increment to 2/4 scored rejections.
- Attempt 6: **`bf85d92d-3810-4dab-84d0-2c3ab703e6c0`**, **FAILED hidden 2.0 s cap**;
  submitted **`ce1ffee43a31d4a1f511c118d7c31c402807f930`**, public **0.791260**,
  fill **0.924091**, note [0213](0213-completion-priority-and-negative-metrics.md).
  Remote **`5d3c2f9068111dc57e16367818b70efe17e9efe2`** matches the full
  tested ordering tree. Workflow **34718929020**, Benchmark
  **21:08:30.39 -> 21:10:08.72 UTC**, **98.32 s**. No hidden score. New
  terminal degree-priority work exceeds at least one hidden time allowance.
  Next revision guards terminal work by original AMD cost, preserving the
  previously validated portfolio on expensive inputs.
- Attempt 7: **`d895008f-037d-4dc8-a7e6-28400a392d03`**, **FAILED hidden 2.0 s cap**;
  tested/uploaded **`7946bcf8d3875f8fb0c520c48095c55d461027f2`**, public
  **0.790705**, fill **0.923840**, note [0214](0214-amd-work-guard-and-larger-factors.md).
  Original AMD work guard and larger sparse-factor scope, all 300 independent
  permutations/counts match. Remote **6a1cdea71bce22ea506c47dd341ec2ec869b0b02**
  matches the entire uploaded ordering tree. Workflow **34720094107**, Benchmark
  **21:32:45.37 -> 21:34:28.18 UTC**, **102.81 s**. No hidden score.
  Original AMD guard does not resolve the cap failure; larger factors are not
  increased again without an independently measured runtime improvement.
- Attempt 8: **`49cad5eb-6004-45f8-b119-35536b77a85b`**, **FAILED hidden 2.0 s cap**;
  tested/uploaded **`c0b9562fe7281dd4955ae6e570a6042530b9eb52`**, public
  **0.790519**, fill **0.923754**, note [0215](0215-indexed-mcs-and-lex-bfs.md).
  Indexed exact-output MCS queue and fixed LexBFS5 / conditional PEO2;
  all required public checks pass. Remote **6fbb5bd23bbf7df4eefef2a56c9cdb70dc970c3e**
  matches the complete ordering tree. Workflow **34721058461**, Benchmark
  **21:53:32.06 -> 21:55:16.20 UTC**, **104.14 s**. No hidden score; queue
  speedup does not resolve total runtime. Stronger earlier-batch work reduction
  is investigated before another upload, rather than increasing final work.

Next [0215](0215-indexed-mcs-and-lex-bfs.md) replaces stale MCS tuples with one
eager indexed item per vertex (all 12 policies exact, 292 public kernel pairs
match, local 9.6x speedup). One fixed reverse LexBFS and conditional PEO2 adds
six further public gains. Independent and fresh complete source agree on all
300 permutations, exact **0.790519213141**, full max **0.807537 s**. 126 active
tests and 64 repeated generated calls pass. Required sandbox all 300
**0.790519 / fill 0.923754**, each full symbolic tuple matches. Ready for
attempt eight, no official grade yet; campaign **2/4 rejected, 5 failed,
0 promotions**, promoted best 07f0e8a2 and best raw de17cd31 preserved.

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

Attempt five **de17cd31-83e4-4d84-9e30-a05d65efc72a** is validating, uploaded
tested source **bf72aacdfabe3985226e13d429e63a1082def316**. No hidden score yet.
Counters remain **1/4 scored rejections, 3 failed workflows, 0 promotions**.

Attempt five subsequently completed and was rejected with a valid hidden raw
gain, **0.842374**. The next [0213](0213-completion-priority-and-negative-metrics.md)
tests structural MCS tie priorities. The 18k/80k/150k control passes all 300
independent/fresh permutations, 126 active tests, 52 repeated generated orders
and sandboxed **0.791290 / fill 0.924110**. Source **50a91b5** is preserved on
`bounded-priority-control`, not submitted. Wider 30k/180k/300k selected policy
reaches exact **0.791259803417**, ten wins/zero losses, independent all-300
helper permutations match; final production checks underway. All 45 missing
generic metric/alpha arms have zero gains and remain diagnostic. Current
campaign **2/4 rejected, 3 failed, 0 promotions**, best promoted 07f0e8a2,
best raw scored de17cd31; no official attempt six uploaded yet.

Final [0213](0213-completion-priority-and-negative-metrics.md) at 30k/180k/300k
passes all 300 fresh complete permutation comparisons, exact 0.791259803417,
126 active tests. One-line completed-visit hard stop preserves all twelve
MCS policies against frozen exhaustive reference and every recorded public
permutation. Final generated 60 repeated orders pass (1 gain/14 ties), maximum
new minimum 0.819756 s. Required sandboxed all 300 **0.791260 / fill 0.924091**,
every actual count matches. Source ready for attempt six; campaign remains
**2/4 rejected, 3 failed, 0 promotions**, current promoted 07f0e8a2.

Attempt six **bf85d92d-3810-4dab-84d0-2c3ab703e6c0** is validating, tested
and uploaded source **ce1ffee43a31d4a1f511c118d7c31c402807f930**. No hidden grade
yet; campaign remains **2/4 rejected, 3 failed, 0 promotions**.

Attempt six subsequently **FAILED hidden 2.0 s cap**, Benchmark 98.32 s, no
hidden score. Campaign **2/4 rejected, 4 failed, 0 promotions**. The next
[0214](0214-amd-work-guard-and-larger-factors.md) preserves the earlier portfolio
when original AMD prediction exceeds 1B and enlarges the new priority scope
to 50k vertices / 1.3M input nnz / 1M factor nnz. Independent and fresh
production match all 300 public permutations, exact **0.790704642081**,
maximum **0.910646 s**. Full release 126 active checks and 64 generated
calls on 16 fixtures pass. Required sandboxed all 300 **0.790705 / fill
0.923840**, every official symbolic row matches the independent replay.
Source is ready for official attempt seven; no grade is claimed yet.
