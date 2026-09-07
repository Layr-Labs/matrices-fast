# 0085 — Alternate-seed sources: extra-relabel tickets

- **Date:** 2026-09-06
- **Score:** 0.826784 → 0.826784 (exact tie, all 300 per-matrix ratios identical)
- **Status:** NEGATIVE locally; reverted.

## Hypothesis

e5310ad retains the top-4 displaced portfolio outputs as PEO chain seeds, but
the extra-relabel lottery tickets (the only other diverse, non-converged
outputs) bypass `consider` and are dropped. Plumbing them into the same
retain protocol adds seed diversity at fixed seed count.

## What changed

`src/ordering/mod.rs`: new `retain_alt_seed` closure mirroring `consider`'s
push/sort/dedup/truncate protocol; called at the two extra-relabel ticket
sites (AMF + AMD) before best-update. Two call sites + closure.

## Result

All 300 per-matrix ratios byte-identical to base. The extra seeds never
displace the retained top-4 (portfolio lottery outputs dominate them) or
their chains stall on the first round. Zero effect, as opposed to zero net
effect — no offsetting moves in either direction.

## Why it lost

The extra tickets are conditioned on well-below-anchor incumbents and few in
number (8–16); their outputs are near-leader refinements, not the far afield
points that make good chain fuel. Diversity has to come from genuinely
different basins (portfolio lotteries), not from more draws near the leader.

## Follow-ups

- Do not wire more near-leader stages (terminal refinements, PEO outputs,
  cutoff refines) into `runner_up` — same reason, established by this null.
- Remaining seed axis: none cheap. Next structural leads are
  bucket-weighted relabel budgets (price with `probe_relabel_budget` first)
  and residual multi-depth prefixes K ∈ {2,4,5}.

## Links

- Research queue: [open-questions](../open-questions.md)
