# Cross-candidate subtree transplant

This is the local experiment formerly numbered 0086. Upstream 0086 denotes
[robust AMD on giants](0086-robust-amd-giants.md).

The submitted bundle built on frontier `996e8d6` and gained **9.10 absolute
dev bips**, 0.826558 → **0.825648**, with a 600k scorer ledger, eight retained
donors, and local stage-7 hygiene. All 300 local rows and 68 tests passed.
Submission `5a0a0a1a` returned **failed**, with no public score, row, or cause.
That exact bundle is not a candidate for resubmission unchanged.

The construction postorders the incumbent elimination tree and forms maximal
disjoint subtree blocks at widths 4096, 512, 128, and 32, minimum size four.
Each portfolio donor supplies an internal order for every block. One exact
symbolic score gives the individual block contributions; strict per-block
improvements are assembled and rescored. Probe assertions verify that the
assembled score equals the incumbent minus the sum of block improvements.
No matrix identity is used: donors, blocks, scores, and gates derive from the
input pattern and deterministic pipeline.

The recorded unbounded production-donor probe moved 87 rows for approximately
35.6 dev bips. A surrogate Python portfolio understated this mechanism because
it supplied different donors. Production artifacts are essential when pricing
a mechanism that consumes the production donor pool.

Each scoring call is charged `n + nnz`; the scorer does not materialize the
filled factor. This ledger does not independently count donor-rank construction,
block sorting, allocations, or other setup, and equal input units do not prove
equal wall time across graph shapes. All such work belongs in a production
cost declaration.

Local interleaved timing gates passed for the submitted bundle, yet hidden
validation failed. The cause remains unknown. Savings on giant rows cannot
fund added work on other rows under a per-matrix cap. A prior score gain is
also not a certificate of available runtime.

The donor loop has a separate, observable truncation effect: exhausting its
ledger can exit the entire width loop before verifying gains from donors
already scored. Reserving verification budget changes that behavior. The
100k screen and below-AMD admission control are recorded in
[0090](0090-transplant-verification-reservation-screen.md).

Exact failed source files and their hashes remain archived by the research
process. The encountered dirty tree, including the original untracked note,
is also preserved in stash `98e7de7a6e4da2e238849725a04c488361232b9f`.
