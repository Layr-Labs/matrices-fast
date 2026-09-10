# 0147 — Constrained-MD design doc (staircase door, paper phase — CLOSED at Gate 1)

## Status

**CLOSED-with-cause at Gate 1 (no target class).** Paper phase only — no
code, no builds, no probes were run for this page; all evidence cited is
already-measured (log/probe/board references given). Gates 2–4 are
answered for completeness but moot once Gate 1 kills.

## Proposal under test

Greedy exact minimum-degree with a protected (constrained-last) vertex
set + live fill propagation — the literature's sophisticated form of
stage-aware ordering (incomplete-ND/constrained-MD hybrids: Amestoy et
al., Pellegrini/Roman/Amestoy, Liu multisectors), as opposed to the naive
border-first/last orderings killed by probe_stage. Constraints from
structure the incumbent itself reveals (etree separators, hub sets —
never family identity), min-degree elsewhere with exact deficiency
tie-breaks (VOL-MDDEF machinery, proven vs AMD), best-of floor.

## Gate 1 — TARGET: no class with real weight survives (KILL)

Requirement: row classes where (a) weight is real, (b) generic machinery
doesn't own them, (c) constrained-MD specifically (not just "another
instrument") has a thesis.

Candidate audit (crown ratios vs AMD, dev corpus):

| class | rows | weight each | crown ratio | status |
|---|---|---|---|---|
| crudeoil lee/family | ~10 | ~0.9% | 0.51–0.78 | OWNED (generic) — off table per order |
| pooling-spp | ~8 | ~0.9% | 0.16–0.63 | OWNED — off table |
| multiplants | 6 | ~0.2–0.9% | 0.41–0.95 | OWNED except stg1c 0.95 |
| stg1c (single 204-hub) | 1 | ~0.2% | 0.9532 | border probe dead 1.01–1.89x |
| rsyn band | 13 | ~0.9% | 0.78–0.95 | MinFill+relabels dead (Phase-0); bounds vacuous |
| gasprod ×2 | 2 | ~0.9% | 0.91 | MinFill dead 2.7–3x; border dead |
| foulds/qapw/knp ties | 4 | ~0.2–0.9% | 1.00 | all instruments dead incl. bounds-backed |
| kissing2/supplychain/emfl/squfl | 5 | ~0.9% | 1.00 | field-dead / all-instrument-dead |
| giants near-tied (transswitch2383 0.9785, unitcommit 0.9764, acopf 0.9737, faclay30/35 0.955/0.952, popdynm 0.949, faclay75 0.9405, gabriel10 0.9285) | 8 | ~0.9% | 0.91–0.98 | **separator graveyard** (see below) |

The only classes with real weight AND headroom are the giant
near-ties — and they are exactly where separator methods are measured
dead: METIS 2.2–4.5x worse on faclay75/gabriel10/acopf/unitcommit
(probe_large + open-questions record), ND catastrophic on networks,
hand-ND 70–13000x on the small analogs, transswitch/powerflow/nd_netgen
covered in-portfolio (best-of includes METIS/Scotch there — losses
discarded silently, crown ratios already reflect their best).

Why constrained-MD cannot escape that graveyard: it IS a separator
method (protect separators, greedy elsewhere). Its fill structure
rhymes with ND-with-AMD-leaves, which loses 2–4x on precisely these
rows. The failure it would need to dodge — separators that don't help —
is the measured outcome, not an open question. No class remains with
real weight: **GATE 1 KILLS**.

What would reopen Gate 1 (falsifiability): a row class with weight
showing separator-shaped headroom — e.g., a new-corpus big row where
METIS beats AMD but the crown ties (unowned separator win), or a bound
gap on a big row for a separator method specifically (all current gaps
are maxcol bounds on rows where ND already died 100x+).

## Gate 2 — SURVIVAL vs the five negatives (MOOT — recorded for completeness)

With no target, survival is vacuous; for the record, constrained-MD
would trip (1) ownership — it attacks rows generic methods demonstrably
lose on, so no conflict, but also no foothold; (2) vacuous bounds —
maxcol bounds can't see its wins (distributional), same blindness as
MinFill; (3) border 9/9 — it is NOT border-first/last (constraint +
greedy is a different search), so that negative doesn't transfer, but
neither does any positive; (4) BFS-diameter — irrelevant (no
stage/diameter claim); (5) literature — the hybrid literature supports
existence, not advantage on THESE rows (its wins are on PDE meshes
where our METIS already wins or ties).

## Gate 3 — COST (MOOT — bound stated anyway)

Constrained greedy ≈ MD pass + deficiency evals on tied sets (VOL-MDDEF
measured this shape affordable on n ≤ 2000 rows, ms-scale) + constraint
detection (etree separators of incumbent: one symbolic pass, affordable;
hub sets: degree scan, trivial). Fits small/medium bands honestly. Big
rows: unmeasurable locally per the margin verdict (ledger-units ≠ wall,
fills unbounded by input gates) — and Gate 1 already closed, so this
stays a bound, never a claim.

## Gate 4 — PHASE-1 EXPERIMENT (MOOT — specified anyway, unbuilt)

Had Gate 1 passed: offline probe, etree-separator-constrained greedy
vs AMD + crown on the surviving class, 0/1 per row (win = beats crown),
pre-declared kill at <3 wins or any worst-row cost above the RR fence
envelope. Single session, machine time only. Not built, not run.

## Links

- Five negatives: ownership scan + bounds trilogy (log), border probe
  9/9 (probe_stage), BFS-diameter (log), literature (this session).
- Cousin mechanisms priced: VOL-MDDEF (deterministic smart ties, nulled
  as candidate, 63 weak wins), relabel lotteries (saturated), MinFill
  family (dead global, alive on residual cores only).
- Reopen conditions: Gate-1 falsifier above, or a new crown changing
  which classes are owned.
