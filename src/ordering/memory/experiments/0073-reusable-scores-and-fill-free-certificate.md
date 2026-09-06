# Reusable exact scoring and a fill-free optimality certificate

Base: 5f68951, extending official df6e3f0. Previous official submission
2ed7e183 failed the hidden 2-second per-matrix deadline. No hidden score was produced.

This revision reuses the previously developed scoring_ws.rs workspace from
matrices-fast-live for all full-pattern scores within leader_order. Its u32
scratch storage, flat etree child lists and unsorted permuted columns preserve
exact column counts while eliminating repeated allocation. State is local to
one invocation. Core scores retain the vendor-backed reference implementation.

After scoring the AMD anchor, compare exact nnz(L) against n + original edges.
Equality certifies no fill. For every elimination completion H, predicted
flops are n + 3|E(H)| + 2 triangles(H). Since H contains the original graph G,
a no-fill ordering attains the lower bound and is globally optimal. Returning
that ordering avoids unnecessary searches without sacrificing the objective.

Validation: 66 active tests pass, including all 1024 five-vertex graphs and
all 120 permutations each, testing the lower bound and every certificate
against vendor scores. A separate 1200-permutation public-corpus differential
check agrees exactly; measured scorer time was 1.597s reference versus 0.722s
workspace. Complete-order timing on 300 graphs was 76.256s before workspace
and 71.630s after workspace; the maximum was 0.699s versus 0.691s. These are
single local series, not hidden-runtime guarantees.

The eight-ticket medium-core portfolio and 2M extra cleanup are unchanged.
The earlier four-ticket official candidate passed all gates but lost to a
concurrent frontier improvement. Two subsequent portfolio variants timed out.
This revision reduces redundant scoring and skips provably useless search;
it does not increase search budgets or infer hidden matrix identity.

Combined certificate + workspace timing: 70.5986s total across 300 complete
orderings versus 76.2564s before either change, a 7.419% reduction in this
series; maximum 0.6857s versus 0.6990s. All 56 generated sandboxed stress
calls pass determinism, bijection, 4GiB and 2-second limits. Candidate maximum
stress wall time is 0.360249s.
