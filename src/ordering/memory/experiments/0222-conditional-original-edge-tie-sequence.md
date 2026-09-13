Final official outcome: **FAILED hidden2.0s cap**,02461af5,workflow34725795538.
Entire remote0f1f033 matches tested52e28c0. Benchmark2026-09-12T23:38:35.9139668Z->2026-09-12T23:40:20.7965061Z,104.88s.
No hidden score. Original-edge search outside terminal dimension was insufficient
for the whole cap. Next shares the final budget with actual late retained-core
work as well. Campaign3 scoredrejects/10 failedworkflows,currentbestfe4f40c0.841858,
our validrawbestbb07f71e0.841858. The body below records historical research/upload.

Baseline bb07f71e is now officially scored and valid: hidden0.841858/fill0.944586,
REJECTED #3 tie,successfulworkflow34724755761. Production activation and fresh
default checks begin on this source. Campaign3/4 rejects/9 failures.

# Conditional original-edge tie sequence on bounded larger completions

Combined sequence is activated for production on officially valid bb07f71e.
Independent joint screen,production-default full300,127 active release tests,
32 production mesh calls and required sandbox all300 build/score passed below. No hidden
score or cap validity for this added sequence is claimed before official grading.

## Problem and resulting ordering

The inherited ordering has already searched several metrics and completed
its incumbent graph. A chordal completion can admit several perfect
elimination orders whose factorization of the original graph has different
fill and predicted flops. Ordinary maximum-cardinality search resolves ties
without distinguishing source edges from fill. This experiment uses the
original incidences to choose among otherwise maximal cardinalities, and
accepts a new elimination ordering only when its actual LDL-transpose flop
count strictly decreases.

The first fixed tie policy, mode3, keeps completion cardinality as the primary
key. Among maximal cardinalities it prefers more remaining original
incidences, then lower completion degree, then the incumbent ordering position.
The visit sequence is reversed to produce an elimination ordering. If that
raw candidate does not strictly improve the actual full-pattern symbolic
flops, the helper returns None, and no further new work runs. If it wins,
at most two ordinary PEO rounds refine the winner, stopping on a no-op and
accepting only further exact flop decreases.

Only a successful first helper authorizes the second fixed policy, mode2.
It uses visited original incidences in place of remaining incidences while
preserving completion cardinality and the same final static ties. It runs
once on the newly completed first winner. A second raw strict improvement
can receive at most two ordinary PEO rounds, under identical caps and strict
acceptance. This is a fixed conditional sequence3 then2, not a four-policy
portfolio, and not repeated until convergence. No failed first raw candidate
can trigger either its ordinary continuation or the second policy.

The second pass addresses a concrete behavior observed in the independent
screen: the first policy changes the completion, so the visited-incidence
policy can discover a different PEO on that smaller completion. It is useful
only when the first search actually changed the original factorization. This
state-dependent authorization avoids another full sweep on the numerous
public patterns where the first candidate ties the incumbent.

## Resource bounds and admission

The new root stage requires original dimension greater than12000 and at most
50000, no more than180000 original input nonzeros, original AMD flops at most
1000000000, and an incumbent that already strictly beats that AMD reference.
Dimension/input/reference gates are checked before another incumbent score.
The helper refuses factor counts over750000 before materializing completion
adjacency. Each raw candidate and its ordinary continuation reuse the same
50000/180000/750000 bounds. The second helper is authorized by first-pass
success, so its starting incumbent also remains below the original AMD bound.

The12000 boundary is the existing full terminal-window class limit. The
production call graph was inspected: the full leader_order and its terminal
profile are called once from order(); nested core refinements use refine_core
and do not recursively invoke the full terminal profile. Thus these added
passes are outside the promoted terminal exchange/five-span class in this
pipeline. Existing bounded nested core work still contributes to total root
runtime, and local helper timings are not a guarantee about that total cost.

The preceding shared-budget change records any eligible sparse terminal
exchange or admitted exact terminal follow-up as final-stage work spent.
A late retained-core quotient walk/search is then skipped. It keeps the late
core available when no terminal work ran. Its all300 public permutations
match the reduced-source control exactly, at local0.791087437358. The two
original-edge passes do not increase the promoted terminal allowances, the
retained-core allowances, or the existing high-flop producer budget.

All admission and tie decisions use structural pattern data, actual accepted
candidate state, or fixed work limits. No corpus identities, dimensions chosen
as lookup intervals, hashes, saved answers, filesystem or network data affect
runtime behavior. The root remains deterministic, with the prior incumbent
and AMD ordering available as fallbacks.

## Queue implementation and correctness

An indexed max heap holds one entry per unvisited vertex. Heap positions and
u128 comparison keys are stored in bounded vectors. Primary cardinality is
updated only by completion adjacency. Secondary incidence labels are updated
by original columns. A pop retires its position, and subsequent updates to
visited vertices are ignored. Key increases repair upward; decreases repair
downward. There are no stale heap entries or adjacency-sized queues.

Low32 key bits contain unique static rank, the next32 contain the incidence
key, and higher bits contain completion cardinality. Mode3 starts each
secondary label at original total incidences plus that column's degree; at
most original total incidences can decrement any vertex, so the common offset
prevents underflow even with directed/duplicate columns. It does not change
comparisons. With180000 input incidences, secondary labels are at most360000;
rank/cardinality are at most50000. The fields fit, and a decrement cannot
borrow from cardinality. Mode2 uses monotonically increasing visited-incidence
labels. Completion cardinality dominates both policies throughout.

Both policies preserve the defining maximum-cardinality choice. The active
independent oracle selects a maximal tuple by scanning remaining vertices,
maintains labels separately, and compares its entire output with the indexed
queue. All undirected graphs through five vertices with multiple incumbents
and all four research modes are checked byte-for-byte, along with perfect
elimination order of the actual completion and the original graph flop floor.
A separate active20-graph test covers dimensions7/17/31/64/127 at four densities,
randomized orders, directed columns, duplicates and diagonal incidences.

Completion storage is the existing flat CSR/u32 representation, capped before
construction. The factor bound implies fewer than1500000 adjacency entries,
about6MB of u32 payload. The heap uses bounded usize arrays and u128 keys.
The first helper releases temporary completion and queue storage before the
second helper constructs its own. No persistent answer cache is created.
These helper estimates are not the whole worker peak memory. All new runtime
implementation uses Rust standard vectors, arithmetic and collections only.

## Independent public screens

The first-policy screen on all300 public reduced incumbents gives exact local
0.790789084587,8 strict wins and no increases versus0.791087437358. Its bounded
helper totals0.301863s,max0.055402 on ARM. Full root ordering with cfg(test)-only
activation matches all300 oracle permutations byte-for-byte, ordering92.596659s,
max0.779093s. Its generated16-fixture/64-call comparison passes repeatability,
bijections and actual AMD floors, with16 score ties.

The conditional second-policy screen evaluates only actual first winners
under the smaller caps, comparing every result with the independent four-mode
research score. It gives0.790535759745,4 further wins,helper0.171607s total,
max0.053888s. Its all300 seed output is then checked against the combined
root pipeline. Every permutation matches byte-for-byte. Combined root local
0.790535759745,4 wins/0 losses/296 ties versus first-only,ordering92.582664s,
max0.801240s,diagnostic wall93.30s. All first-policy gains remain strict, so
joint versus the shared-budget control is8 wins/0 losses/292 ties.

The substantial further reductions include crudeoil_lee4_10 from180946666
to178533850 and arki0013 from154312768 to150475002 predicted flops. Names
are recorded only as public evidence and are absent from runtime selection.
Full public evidence is archived under0221/0222 memory tables. These research
numbers are not the future sandbox claim or a hidden score. Structured mesh
stress and production-default release/sandbox checks are pending next.

## Prior outcomes, frontier and attribution

Current external promoted frontier is newjordan fe4f40c,hidden0.841858,
fill0.944586,immutable256152b3da9b08028ab82c90f373c9e64429c18f,successful
workflow34721278191. Its full public note was read as untrusted data and its
active terminal settings verified against that immutable source. The inherited
536870912 exchange/four PEO/five-span schedule is credited to newjordan, who
will be listed as upload coauthor. This combination needs its own grade.

Our original promoted07f0e8a2 has hidden0.842377. Previous raw scoredde17cd31
has0.842374 and was rejected below the one relative-basis-point promotion
floor. Our current raw scored best isbb07f71e at0.841858,matching the global
frontier; our own promoted best remains07f0e8a2. All are preserved separately. Later wide priority/LexBFS sources and
reduced2a68edc7 failed the hidden2.0s cap and have no hidden score. Candidate
bb07f71e,tested2068133ecf5915f52ffef8606f6d073ac199f141,remote0f006f4d36f270780a01284efb8632794717d220,
entire ordering tree verified,shares the terminal/core late budget and passes
workflow34724755761. Official hidden0.841858/fill0.944586 tiesfe4f40c,so it is
scored REJECTED #3. Benchmark23:14:14->23:23:26UTC,552s by job timestamps.
The added sequence is activated on this passing source and needs its own grade.

Before the next upload,the authorized fresh campaign has3 scored rejections,
9 failed workflows,and0 own promotions. The user requested continued work
until more than three submissions are rejected; the stopping condition is
4 official scored rejections, with failures tracked separately. Public and
hidden scores are never treated as directly comparable improvement deltas.

## Submission requirements

Only src/ordering/ is edited. Trusted harness/scoring,workflow,corpus,Cargo
manifests and dependency declarations remain unchanged. Runtime code has no
environment,filesystem,network,clock,corpus membership or saved-answer access.
Diagnostic caches and control cells are cfg(test)-only. Actual production
activation will default to the new conditional sequence and receive fresh
release tests,generated checks and the required Yukon sandbox build/score.
Every dimension/input/AMD/candidate-flop tuple must match the independent
oracle,not just the rounded aggregate. Actual JSON and final300-row table
will be archived before upload,then source committed/frozen and remote tree
verified. No hidden improvement or time-cap compliance is claimed until the
official result. Model metadata will be GPT 6,harnessCodex,reasoning high,
coauthornewjordan. No authentication token appears in this note or evidence.

Structured mesh diagnostics:8 new generated fixtures/32 root calls passed in24.52s. Each arm repeats byte-exactly,each result bijective/at or below AMD,and joint never exceeds control. 5 wins/3 ties,5 first-policy acceptances,2 second-policy acceptances. Flags repeat and every second acceptance requires first acceptance; when first did not accept,full output equals control. Actual structural graphs have128x128 or96x192 cells with diagonal/local/hub edges. These are independent generated examples,not stored runtime cases. Evidence archived; production-default checks pending.

Production activation: original-edge module, bounded candidate helper and two-policy post-core helper now compile in release. Root default enables first mode3 and conditional secondmode2. Test override cells are Option<bool> defaultNone and mirror literal productiontrue; full-pipeline probe no longer disables production when no research flag is provided. Wide priority/LexBFS and medium100M/64 remain test-only/disabled. Release checks are running; no upload yet.

Fresh production-default release suite passed:127 active tests,77 ignored,82.42s. This run includes the newly compiled original-edge helpers and literal enabled production defaults; no forced original-policy flags are used. Full300 default pipeline comparison to the joint oracle is now running.

Fresh production-default300 check passed:every complete output permutation matches the combined bounded oracle byte-for-byte,exact0.790535759745,0 differences against joint research seeds. Eight first-policy/four second-policy acceptances; every second flag has the first flag. Ordering92.150883s,max0.797746s,diagnostic92.84s ARM. No original-policy environment overrides were used; testOptionNone mirrors true production defaults. Final production mesh and required sandbox follow.

Final production mesh diagnostics passed32 root calls across8 fixtures in24.41s. Every actual flop/AMD/dimension/input/acceptance-flag result matches the research fixture run,5 improvements/3 ties,2 second acceptances. Bijections/AMD floors/byte-exact repeats/closed-first output equality all pass. Maximum ordering0.915230s,largest added time0.092985s on paired ARM minima; no hidden-platform timing guarantee. Required sandbox running; no thirteenth upload yet.

Final pre-upload verification:required Yukon sandbox build/score completed on all300 public patterns,actual primary0.790536,fill0.923722. Every dimension,input,AMD-flop and candidate-flop tuple matches the independent default-production screen exactly. ScoreJSON and300 final rows archived. Release127 passed/77 ignored,production300 P byte-exact oracle,production mesh32 calls/8fixtures passed5 wins/3 ties/2 second acceptances. Changes confinedtoordering. Model GPT 6,harnessCodex,reasoning high,coauthornewjordan. Combined hidden cap/score still unverified; officialbaselinebb07f71e is valid0.841858. Campaign3/4 scoredrejections,9 failedworkflows,0 ownpromotions. Ready for upload.

Queued official02461af5-12a8-43a7-bba6-c60d1f2526dd,tested/uploaded52e28c0c62582960403813e202f3c26881250118,coauthornewjordan,note14KiB. Actuallocal0.790536/fill0.923722. Hidden grade pending; no promotion/cap compliance claimed. Campaign3 scoredrejects/9 failedworkflows.
