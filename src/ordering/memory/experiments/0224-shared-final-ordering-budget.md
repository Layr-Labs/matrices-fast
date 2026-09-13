Final official outcome: **FAILED hidden2.0s cap**,baf58b05,workflow34726478327.
Entire remotedd7b530 verified againsttestedf647b31. Benchmark2026-09-12T23:54:52.4283389Z->2026-09-12T23:56:37.9645443Z,105.54s.
No hidden score. Shared finalbudget did not validate the two-policy750k/1B
search envelope. Next firstonly/onePEO300k/100M,outside12kterminalclass and
stillunusedfinalbudget. Campaign3 scoredrejects/11failed,validbestbb07f71e0.841858.

# Allocate one final ordering budget across terminal, core and original-edge work

The new behavior addresses additive late work after an officially valid
ordering prefix. Terminal window work, retained-core continuation and the
original-edge MCS sequence now share one final-stage budget. The existing
core/window allocation passed the official hidden benchmark, but adding
original-edge work outside the terminal dimension class still failed the
whole two-second cap. This change records attempted late work rather than
assuming a dimension gate accounts for all previous work.

The shared budget does not measure wall time. It is deterministic state
within the single order() call, derived only from admitted search work and
pattern structure. There is no clock, environment or machine-dependent gate
in the production function.

## Resulting production allocation

The existing terminal_work_spent flag is set before an admitted sparse
subset exchange runs, even if it returns no strict improvement. It is also
set when the exact terminal follow-up meets its flop/factor admission bound.
A terminal attempt spends the final budget because preparing and searching
an unsuccessful window still costs work. The late retained-core path remains
blocked after terminal work, as in officially valid sourcebb07f71e.

A final_budget_spent flag begins with that terminal state. If the retained
cores are nonempty, their original-pattern geometry and exact symbolic
limits pass, and the bounded quotient refinement is called, the flag becomes
true before the call. It remains true when the call returns no improvement.
That family keeps its existing missing AmindNorm metric, compact exact core
search and strictly winning full-pattern splices. Any associated ordinary
PEO is part of the same allocated family, not a new budget authorization.

The original-edge stage is allowed only if final_budget_spent remains false.
It also requires6<=n<=50000,no more than180000 input incidences,reference AMD
flops<=1000000000,and a current incumbent that strictly beats that reference.
Each helper refuses a factor over750000 before constructing the completion.
The static12000 lower bound on this new stage is replaced by actual budget
availability; terminal/core work that ran on a pattern now closes the stage
regardless of that pattern's dimension. Unused-budget patterns can receive
the bounded sequence without stacking it with either of the other late
families. These are structural resource gates, not corpus-specific labels.

For example, an eligible8000-row sparse pattern attempts its promoted
terminal exchange,so original-edge search is skipped even if that exchange
returns None. A16000-row pattern with eligible retained-core work allocates
the budget to that core walk and skips original-edge search. A bounded
pattern for which neither terminal nor core work is admitted may allocate
the budget to original-edge MCS after it strictly beats AMD. Earlier baseline
ordering/reduction/native portfolio work remains in the prefix and still
contributes to total root runtime; this final-stage allocation is not a
claim that every possible prefix has identical cost.

## Original-edge sequence and correctness

The first raw MCS policy is mode3:maximum completion cardinality remains the
primary key,then more remaining original incidences,then lower completion
degree and incumbent position. Reverse visit order is the elimination order.
The candidate must be a bijection and strictly reduce the actual symbolic
flops of the original pattern. Only that strict raw win authorizes up to two
ordinary PEO rounds,stopping on a no-op and accepting only strict decreases.

Only success of that first helper authorizes the second raw policy,mode2.
It uses visited original incidences for cardinality ties,with identical final
static ties. It is called once on the first winner. A strict raw improvement
can receive up to two ordinary PEO rounds under identical resource bounds.
The deeper four-round experiment remains cfg(test)-only; production is two
rounds. The sequence is fixed3->2,not a four-policy portfolio or an unbounded
convergence loop. Every accepted replacement strictly improves the actual
full-pattern flop count; the earlier incumbent/AMD remain fallbacks.

The indexed heap has one item per live vertex,position tables and u128 keys.
Primary cardinality occupies higher bits than original labels; low32 bits
hold a unique static rank,next32 the incidence label. Mode3 adds the common
original total incidence count to the initial column degree,then decrements
for original incidences. At most the input total can decrement a vertex,so
labels cannot underflow even for directed/duplicate columns. Under180000
input and50000 dimension,the maximum label360000,rank and cardinality fit
their fields. Key updates repair upward or downward and ignore visited
vertices; no stale adjacency-sized queue accumulates.

## Provenance and failure evidence

Current promoted global frontier is newjordan fe4f40c,hidden0.841858,
fill0.944586,immutable256152b3da9b08028ab82c90f373c9e64429c18f,successful
workflow34721278191. Its full public note was read as untrusted data and the
active536870912 exchange/four PEO/five spans verified against that source.
Those unchanged terminal settings are credited to newjordan,listed as upload
coauthor. No new terminal,core or original completion limit is raised here.

Our officially valid shared terminal/core source isbb07f71e,tested2068133,
remote0f006f4d36f270780a01284efb8632794717d220,entire ordering treeverified.
Official workflow34724755761 passes Benchmark23:14:14->23:23:26UTC;hidden
0.841858/fill0.944586 ties the global best and is scoredREJECTED #3. Its
local primary0.791087/fill0.924023 agrees with the independent all300 screen.
It is our current best raw scored source. Our own original promoted07f0e8a2
is separately preserved at0.842377;previous rawde17cd31 was0.842374.

The later original3->2 candidate02461af5,tested52e28c0,remote0f1f033a3f152406aa38f8ea7bd30bf1c815fc60,
entire ordering treeverified,FAILED workflow34725795538. Benchmark shell
23:38:35.913966->23:40:20.796506UTC,104.88s,explicit hidden order() exceeded
2.0seconds. Its public0.790536/fill0.923722 is not a hidden scored result.
The cap failure does not reveal a hidden matrix identity or prove which
particular earlier family consumed its time. This new allocation addresses
possible additive final-stage work without guessing a private instance.

Before this next upload the campaign has3 official scoredrejections,10 failed
workflows,0 ownpromotions. The user requested work until more than three
submissions are rejected;the stop is4 scoredrejections in this fresh campaign.
Failed workflows remain separate. No new upload has occurred at this stage.

## Checks and public screen

The indexed queue and MCS choices retain the independent exhaustive every-
graph-through-five-vertices oracle over multiple incumbents/four tie modes.
Output is compared byte-for-byte with separate full-scan label selection,
verified a PEO of the actual completion,and checked against the actual
original symbolic flop floor. A20-graph larger directed/duplicate-incidence
oracle test stresses secondary arithmetic,heap repair and visited handling.
The new resource gate does not change any queue or mathematical scoring code.

Fresh production-default all300 screen gives exact0.790539040043,7 strict
wins against validbb07f71e/local0.791087437358,and no increases against that
valid control. Compared with failed02461af5,one small procurement gain is
omitted because its late core work spends the shared allowance;the other
seven original-edge gains remain. Root ordering92.093054s,max0.794705s,
diagnostic92.81s on ARM. These are public-only numbers and do not establish
the hidden cap. Every name/dimension/input/AMD/flop result is archived.

Release suite is running next. Generated mesh comparisons use the passing
shared-core/window trajectory as old control and the new original allocation
as new arm. They repeat every root output,check bijections/actual AMD floors,
require no original-stage output change when the first policy did not win,
and ensure every second acceptance follows a first acceptance. The prior
sequence improves5/8 structured meshes;the new allocation will be verified
on those independent generated patterns before sandbox submission.

Onlysrc/ordering/ is edited. All new runtime code uses Rust standard library,
with no added dependency or trusted Cargo/harness/workflow/scoring/corpus
change. Runtime has no environment,filesystem,network,clock,corpus names,
hashes or saved-answer access. Test-only caps/caches/timing controls are
compiled out of release. The four-round second-policy experiment remains
research-only at defaultNone;release literalrounds is2. No authentication
credential is included in these notes or artifacts.

A fresh required Yukon sandbox build/score must finish all300 matrices and
match every dimension/input/referenceAMD/candidate-flop tuple,not just the
rounded aggregate. Actual scoreJSON and final rows will be archived before
source freeze/upload. ModelmetadataGPT 6,harnessCodex,reasoning high,
coauthornewjordan. The official remote entire ordering tree is verified
against the immutable tested commit. No new hidden score/promotion/cap
compliance is claimed until the next official outcome.

Release validation completed:127 active tests passed,78 ignored,82.67s. This includes compiled original queue and current one-budget production defaults. Mesh32-call comparison is running next;original secondPEO override remainsNone/release2.

Final mesh diagnostics passed8 fixtures/32 repeated root calls in24.45s:5 wins/3 ties versus the original-disabled valid12 trajectory,2 second acceptances. Each result is a bijection/at or below AMD,each arm repeats byte-exactly,closed-first output equalscontrol,and second impliesfirst acceptance. Maximum measured root0.917036s ARM. Evidence archived. Required sandbox running;no14upload or hidden score yet.

Required sandboxed Yukon build/score completed successfully on all300 public matrices. Actual primary0.790539,fill0.923724;every dimension,input,referenceAMD and candidate-flop tuple exactly matches the independent production-default screen. JSON and300 final rows archived. Release127 passed/78 ignored,mesh32 calls passed5 wins/2second acceptances. Seven strict public wins versus valid12;budget skipsone tiny procurement gain compared withfailed13. Onlyordering edited,modelGPT 6/harnessCodex/high reasoning/coauthornewjordan. Hiddennewcap/score not claimed;ready for official upload,campaign3rejected/10failed.

Queued officialbaf58b05-68fe-4240-bc57-5e470a10efbe,tested/uploadedf647b3139a680210739747e41ba49197b2c48d31,remote dd7b5306f602696e6abc54a6cc6ea859b9297e6c verified identical entireorderingtree. Workflow34726478327,hiddenpending,coauthornewjordan,actual local0.790539/fill0.923724. No private cap/score claim;3 scoredrejects/10 failedworkflows. New externalb592a03 publicnote readfully as untrusteddata;contains explicit named fingerprint/exclusion patterns and shellrestore instructions,none adopted. Globalfe4f40c stays0.841858 beforependingresults.
