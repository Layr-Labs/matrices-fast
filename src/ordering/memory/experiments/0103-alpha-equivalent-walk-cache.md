# Coalescing identical alpha walks while preserving replay

Effort: high. Remote-only tests and profiling; no local builds.

## Evidence and hypothesis

Experiment0102 validated symmetry-based canonical permutation, but its
roughly25% helper speedup translated into only about1% full-probe savings
and did not resolve the public timeout. Portfolio generation remains a
large measured cost, including over a second on several slow public cases.

The light quotient-metric block generates ten variants at four alpha
settings. Reading the pinned quotient workspace establishes that alpha
is used only to set a dense-degree threshold at initialization. The sets
of vertices exceeding those thresholds are nested. Equal cardinalities
therefore imply identical sets, identical workspace initialization and,
for a fixed variant/aggressive flag/pattern, identical elimination walks.

## Critical semantics

Deleting duplicate candidate slots would not be a safe optimization:
flush_batch replays every result through a runner-up ledger, and a repeated
incumbent can enter that ledger. Later refinement can observe this.

Instead each alpha slot remains queued in exactly its old position.
Equivalent slots share one Arc/OnceLock result within a single variant.
Whichever equivalent alpha initializes first produces the same result;
every slot then clones that immutable result and undergoes the original
validation, scoring and ordered replay. Results are never cached across
matrices or across metric variants.

## New costs and risks

There is an O(n) permutation clone per replay slot, small cache-key scans,
and synchronization when multiple worker threads reach the same cell.
Stored results add O(classes*n) transient memory. Unique classes receive
no elimination-work savings. Wall-time benefit therefore needs measurement,
especially with the existing four-thread queue on a two-vCPU lab runner.

## Isolated validation

Remote run34166268728 compares validated linear-permutation source against
the same source plus this cache, both on e88316d. Only candidate has the
cache; the separate lab now accepts independently pinned baseline/candidate
patches. Trusted challenge files and dependencies are unchanged.

Tests compare concurrently initialized cached results against uncached
walks on paths, hubs, and dense-core graphs at several sizes, and verify
that distinct dense sets receive distinct cells. Full300-matrix public
counts and timing probes must also agree before calling it output-identical.
Run34166268728 failed before candidate compilation: patch transfer corrupted
UTF-8 alpha characters in a context hunk. Baseline measurements completed,
but there is no candidate test or timing result. Reuploaded with byte-correct
base64 and verified the decoded remote SHA256 equals the intended local
patch hash. Reverse patch applicability and scoped diff checks passed.
Corrected run34167471391 completed successfully. Both trusted public graders
passed all300 matrices, score0.806560 and fill0.930670. All300 full-probe
COUNTS rows agree exactly. Cache tests2passed, including252 concurrent cached
versus uncached permutation comparisons. Baseline symmetric tests2passed.

Full probe163.75s to161.55s (~1.34% less); worst1.822s to1.773s. The summed
rounded portfolio phase times fell95.8953s to93.2548s (~2.75% less). This is
a modest same-run improvement, not proof of universal tail safety.

The runner used2logical CPUs on Intel Xeon6973P-C, unlike earlier EPYC7763
runs. Both arms passed the public time cap. The large cross-run time change
and disappearance of the earlier timeout cannot be attributed to this cache.
Keep the validated stack as baseline for the isolated MINL epoch correction
in experiment0104, dispatched as remote34168819206.

Source diff checks and lab shell syntax checks passed. No official
submission was made: a pure speed optimization needs a separate objective
improvement to meet the leaderboard's strict promotion threshold.
