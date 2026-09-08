# Isolated MINL neighborhood epochs after exact-work reuse

## Result

Remote34168819206 completed: both arms2targeted tests passed; all300 public
COUNTS rows exactly unchanged, score0.806560. Full probes214.38s/214.21s,
worst2.450s/2.473s. Both capped graders failed crudeoil_lee4_10 at2s on the
two-logical-CPU AMD EPYC9V74 runner. No objective gain; not submitted as an
improvement. The correction remains in the test-only screen baseline for0105.

Prepared 2026-09-07; no local builds, tests, benchmarks or candidate execution.

## Question

Does repairing stale neighborhood membership improve the completion descent
under its existing operation budget? The prior direct submissions of this
fix timed out without a score. They did not measure its public-corpus benefit.
This experiment separates that question from the adaptive clique lookup
change preserved in experiment0099.

## Source evidence and correction

The current MINL scan groups edges by coarse degree bucket and lower endpoint.
A vertex can therefore be revisited after another endpoint has removed an
incident edge. Reusing the vertex number as the neighborhood stamp can leave
that deleted neighbor marked, unless an intervening group happened to
overwrite it. The resulting intersection is not necessarily the current
N(u) intersection N(v), which the chordality-preserving deletion test needs.

Each rebuilt neighborhood now receives a fresh epoch. Its vertices alone get
that epoch. Existing in-group deletion still clears the removed neighbor.
On counter wrap, all marks are cleared and that work is charged. Ordinary
rebuilds keep the same row-length logical charge as before.

There are no added passes, wider gates, extra realization candidates, modified
score calculations, or budget increases. Correcting the intermediate graph
can change the descent trajectory, so final score monotonicity against the
old implementation is not assumed. The outer exact scorer remains unchanged.

## Validation plan and current state

Two targeted tests cover stale-neighbor revisits and epoch wrap/charges.
They are prepared, not executed. Scoped git diff --check and reverse patch
applicability passed. The candidate-epoch.patch lab artifact preserves this
change separately while run34167471391 finishes the preceding alpha-cache
comparison. No duplicate job and no new official submission was launched.

The cache subsequently passed in run34167471391: two targeted tests and
identical300 public COUNTS rows, both trusted graders passing. This experiment
is now dispatched as run34168819206, private-lab commit7801e15425e792b1276b20f2d67921885d17eceb.
Both source refs are e88316d. Baseline patch18d019f62dfa83531447da0d611fdd9104f68f7dffb059e40e9da2b9e92a977f
contains validated linear permutation and cache; candidate patche72856333246913a6518e7e2822cafc901dcdc6e11e69a6f43eb49a5f9a36c97
adds only the epoch correction. Remote decoded byte hashes verified after upload.
Tests, full public grader and timing probes are pending; no score claimed.
Inspect every changed COUNTS row, runtime distribution and cap failures before
any official submission decision.

## Further source-level interpretation while the remote run is active

Rebuilding the old stamp writes every current neighbor, so its stale matches
form a superset of the true common neighborhood, not a subset. A superset
containing a non-clique cannot itself be a clique. Thus the observed stamp
defect can falsely refuse a valid deletion; it does not by itself certify an
invalid deletion. Removing that refusal can alter later greedy choices, so
this still does not imply improvement over the old descent's final score.

A proposed local-star alternative also has an exact interpretation: with u
fixed, edge uv is deletable precisely when v is simplicial in the graph induced
by N(u). Deleting another edge at u removes one vertex from that induced graph,
so already-simplicial candidate neighbors remain simplicial. However, the
current completion.rs already calls minl_watch::watcher_minimalize, whose
four-cycle witnesses wake blocked candidates after support-edge deletion.
Source inspection therefore rules out presenting a simple event-driven
star-retest mechanism as a new missing algorithm family. No additional pass
or implementation was added on that rationale.
