# 0166 — Budget-side levers (relabel AMF ceiling, bucket-weighted reshaping): both NEGATIVE

- **Date:** 2026-09-14 (pre-dawn round; score probes only, timing columns
  load-invalid and unused — the flock quiet window discipline)
- **Score:** 0.789841 → **0.789841** (lever 1) and → **0.790294** (lever 2,
  +4.5e-4 WORSE); both reverted, base re-confirmed exactly (0.789841, 130
  tests).
- **Verdict: both NO-GO — the original well is dry on the budget side too;
  ride the held 0163 bundle + port rounds.**

## Lever 1: RELABEL_AMF_MAX_NNZ 200k → 400k

The open-questions note ("130k–400k band") predates 0008's raise to 200k —
the remaining move is 200k→400k. Score IDENTICAL (zero movers): the band's
dev rows (gams05 n 17 364 / nnz 252 910, …) receive only
`500 000/252 910 = 1` extra AMF pass under the shipped budget law and no
single pass wins there. The 0152 "timing death" context was about the
lifted-core family, not this plain band — but there is no value here either
way.

## Lever 2: gt_10k relabel budget 500k/36 → 800k/48

The 0007 bucket-weighting is already the shipped shape (500k/36 vs 400k/30
vs 300k/24 — the open-questions item is stale). Pushing the gt_10k tier to
800k/48 scores **0.790294 = +4.5e-4 WORSE**: with strict per-pass acceptance
the added restarts cannot lose at the site, so the entire regression is
downstream basin shift — more early lottery wins change which incumbent the
cascade/exchange chain polishes, and the polished result is worse. The
restart budget's optimum is where 0007 put it; the family's remaining value
(0003's law: wins land in the first handful of restarts) is already
exhausted. Same mechanism class as 0165's BRKGA receipt.

## Consequence

Every cheap lever from the tree's own winner-class (budget moves 0007/0008/
0011) is either shipped (bucket weighting, AMF 200k) or measured negative on
the current tree (this page). Combined with 0162/0164/0165, the original
stream's remaining EV sits only in: kernel-core-fed exact search (0162's
documented path), the full flow-cutter reference port (0164, expectation-
zero), and work-priced tail-exchange re-armament (0163's device at higher
doses with a gate that prices the near-cap class).
