# Original-edge ties on larger bounded chordal completions

Status: research-only full-pipeline check; production activation awaits the
official result of the shared-terminal/core-budget candidate bb07f71e. This
note is preparation for a subsequent independently checked submission, not
an upload claim or a hidden-cap guarantee. No thirteenth submission exists
at this stage.

## Ordering change

Maximum-cardinality search visits a vertex whose number of already visited
completion neighbors is maximal. The new pass preserves that primary key
and changes ties using the original pattern. Among equal cardinalities it
prefers vertices with more remaining original incidences. A fixed final tie
prefers lower completion degree and then the incumbent ordering position.
The result is reversed to form an elimination ordering. It is a single fixed
policy, mode3; production will not evaluate the four-mode research portfolio.

The original pattern separates source edges from fill introduced by the
incumbent. A chordal completion can have many possible perfect elimination
orders. Choosing among its maximal-cardinality ties using remaining source
incidences is a general heuristic for finding an elimination order whose
completion of the original graph omits more unnecessary fill. This changes
no matrix values and uses no matrix names, hashes or stored solutions.

After computing one raw candidate, the helper verifies bijectivity and its
actual original-pattern symbolic LDL-transpose flop count. It accepts only
a strict decrease. Ordinary completion PEO refinement, at most two rounds,
is authorized only by that strict raw decrease and stops on a no-op. Every
replacement in that continuation must also strictly decrease the actual
symbolic score. If raw original-edge MCS does not win, it returns None and
no continuation runs. The incumbent remains the fallback in that case.

## Resource admission

This pass is restricted to original dimension greater than12000 and at most
50000, with at most180000 original input nonzeros. The12000 boundary is the
existing terminal-window class limit, so the pass cannot stack with the
terminal exchange and five-span work on the same matrix. The original AMD
flop reference must be at most1000000000, and the accepted incumbent must
already strictly beat that AMD reference. Eligibility is determined from
actual scores and structural resource caps, never a corpus identity.

The helper refreshes the exact factor count and refuses a completion with
more than750000 factor nonzeros. Completion reconstruction independently
checks input and factor caps before materializing adjacency. It also checks
the original column-pointer shape, monotonicity, last pointer, original row
bounds, and supported modes. Ordinary PEO after a raw win uses the same
50000/180000/750000 limits. This is smaller than the failed wide priority and
LexBFS prototype's1300000 input/1000000 factor bounds. No terminal window,
core quotient metric or high-flop producer allowance is increased here.

The root guard first checks dimension/input/AMD limits before asking for an
additional exact incumbent score. The raw construction is called once. Work
is independent of elapsed time, filesystem, environment, network, machine
identity or public corpus membership. Production order() remains a pure
function of its pattern with deterministic fixed traversal and tie choices.

## Indexed queue and bounded arithmetic

The queue contains exactly one entry per unvisited vertex. It stores a heap,
a position table and one u128 comparison key per vertex. Pop retires the
position with usize::MAX; updates to visited vertices are ignored. Every
completion-neighbor incidence adds to the primary cardinality key. Every
original column incidence updates the secondary key. Key increases repair
upward and decreases repair downward in the indexed heap. There are no stale
entries and no duplicate heap records that grow with fill.

The low32 bits contain a unique static rank. The next32 bits contain the
original-incidence key. Higher bits contain completion cardinality, which
always dominates the tie rule. For mode3 the secondary key starts at the
original total incidence count plus that vertex's column degree. The total
number of decrements is at most the original incidence count, so the global
offset prevents underflow even for asymmetric columns or repeated incidences.
That constant offset is common to all vertices and cannot change their
secondary comparison. With at most180000 input incidences, the initial
secondary value is at most360000; rank and cardinality are bounded by50000.
All fields fit their assigned widths. A decrement cannot borrow from the
cardinality field. For symmetric source columns the secondary difference is
exactly remaining original degree; raw incidences retain deterministic meaning
for the directed/duplicate inputs accepted by the existing scoring contract.

Completion adjacency is the existing flat CSR/u32 representation. The capped
factor produces fewer than1500000 off-diagonal adjacency entries, about6MB
for the flat u32 storage. The queue's heap and positions use two usize arrays,
and its key array uses u128. Static priorities and leading scores are temporary
bounded vectors. This helper uses Rust standard collections/vectors and adds
no dependency. Its storage is released after the pass; no persistent answer
cache survives order(). These per-helper estimates are not a claim about
the entire inherited worker's peak memory on the hidden corpus.

## Independent correctness evidence

The research implementation is checked against an independent maximum-tuple
selection oracle that scans all remaining vertices and maintains labels
separately. The active exhaustive completion test enumerates every undirected
graph through five vertices, uses multiple incumbent orders, and compares all
four original-incidence modes byte-for-byte with the reference. It checks
that reverse search is a perfect elimination order of the actual completed
graph and that original-pattern symbolic flops do not exceed the incumbent.

A separate active test uses20 larger graphs: dimensions7,17,31,64 and127 at
four densities, randomized incumbent orders, and directed columns with
duplicates and diagonal incidences. All four modes match the independent
oracle byte-for-byte, remain PEOs of the original completion and obey the
actual original graph flop floor. Incidence perturbations specifically test
secondary-key arithmetic and queue repairs. These correctness checks passed
in the127-test release suite of the prior budget source. The production
activation and default behavior will receive their own release checks before
any upload.

## Public research evidence and provenance

The direct larger-class screen starts from all300 independently computed
public incumbent orderings of the reduced source. That control's exact
primary score is0.791087437358. It is also byte-exact with all300 public
orderings of the shared-budget candidate2068133, verified by comparing the
full cache files. One bounded mode3 pass plus strict-win ordinary PEO reduces
the local primary to0.790789084587 with8 strict wins,20 structurally eligible
cases, and no flop increases. The measured helper totals0.301863 seconds,
maximum0.055402, on this ARM macOS machine. Those are warm helper measurements
and a public screen, not this future candidate's sandbox claim or a private
score. Full-pipeline and default-production comparisons follow below.

The eight public gains include larger sparse pooling/procurement patterns,
crude-oil optimization patterns, a nuclear instance, and arki0013. None is
recognized in runtime code; the screen records names solely as public
experimental evidence. The substantial gains have incumbent factor counts
between559733 and695571, within the fixed750000 factor cap. Other strict
gains occur at108978 through471330 factor nonzeros. The admission boundaries
are resource limits rather than lookups for those instances.

The current promoted global result is newjordan fe4f40c at hidden0.841858,
fill0.944586, immutable256152b3da9b08028ab82c90f373c9e64429c18f, official
successful workflow34721278191. Its verified terminal schedule is retained
and credited; newjordan will be included as coauthor of any upload inheriting
those settings. Our best raw scored candidate is de17cd31 at hidden0.842374,
rejected below the one relative-basis-point promotion floor. Our own original
promoted07f0e8a2 at0.842377 remains separately preserved.

Recent failed wider priority/LexBFS candidates never received a hidden score.
Candidate2a68edc7 also failed the hidden two-second cap despite removing those
passes. Candidatebb07f71e shares the terminal and late core budget and is
currently awaiting its official result. This next pass is outside that
terminal class and will not be uploaded on a baseline that remains known
invalid. The campaign currently has2 scored rejections/9 failed workflows;
the authorized stopping condition is4 scored rejections, with failures
tracked separately.

## Scope and required checks

Only src/ordering/ is edited. The runtime implementation uses Rust's standard
library only. Dependency declarations, corpus, trusted scoring and build
scripts, workflow, Cargo manifests and harness are unchanged. Public caches,
names, timing tools and mode controls exist only under cfg(test) or memory
evidence and do not enter production. No token or authentication credential
is present in this note. The actual model metadata will be GPT 6, harness
Codex, reasoning effort high, with coauthornewjordan.

Before upload, the full300 production-default results must equal the direct
helper oracle's exact permutations and symbolic score. Required release
checks and generated repeated root calls must pass. A fresh Yukon sandbox
build and run must score all300 patterns and agree on every dimension/input/
AMD/candidate-flop tuple with the independent screen. Its actual score JSON
will be archived. The source will be committed and frozen before upload;
the remote entire ordering tree will be checked against that tested commit.
No hidden improvement or cap validity is claimed until official evaluation.

Full-pipeline research screen completed:300 exact output permutations match the independent single-policy/strict-win-continuation oracle byte-for-byte. Exact local0.790789084587,8 wins/0 losses/292 ties versus budget control0.791087437358. Root ordering92.596659s,total diagnostic93.31s,max0.779093s ARM. These outputs used the cfg(test)-only activation while production remained unchanged.

Research generated diagnostics:64 repeated root calls/16 structural fixtures passed in29.34s,all bijections/AMD floors/repeats valid. 0 wins/0 losses/16 ties,maximum minimum time old0.872351s/new0.872321s ARM. Shared core/window budget and promoted windows identical in both arms; medium fence/wide MCS/Lex disabled. Production activation and default-release/sandbox checks still await the budget source official result.
