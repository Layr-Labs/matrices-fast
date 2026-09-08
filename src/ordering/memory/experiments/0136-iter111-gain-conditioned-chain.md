# 0136 — iter111: gain-conditioned deeper rebuild on the FINAL_REFINE chain

- **Date:** 2026-09-08
- **Base:** `7257386` (submission `c438774`, hidden 0.849130 — darthweenies' lean
  terminal five-descent), local dev **0.804919** reproduced exactly on this box.
- **Result:** local **0.804719** (−2.00 bip), fill 0.929685 → 0.929567.
  **12 movers / 0 worse.** 71/71 tests.

## Hypothesis

`4f0719d` named "gain-conditioned deeper rebuild" as the next step and left it
unbuilt. The FINAL_REFINE block (iter108/110) refines the *shipped* incumbent
and then runs exactly ONE chained rebuild round, gated on round 1 having paid a
strict gain. The stage-5 terminal chain, by contrast, already runs THREE rounds.
So the terminal chain was strictly shallower than the mid-pipeline chain it was
modelled on.

Rather than pin a third round, replace the fixed depth with a loop: keep
rebuilding while each round pays a strict gain. Depth is then bought only on
rows that keep converting, and the loop exits after one integer comparison
everywhere else.

## Implementation

`src/ordering/mod.rs` only. After the existing round 2, a
`for extra in 0..FINAL_CHAIN_EXTRA_ROUNDS` loop that breaks unless the previous
round strictly improved. Each iteration re-postorders the current incumbent,
recomputes counts/parent, and runs `subtree_refine` with `cfg.round = 2 + extra`
(seed diversification), `budget` 4M (2M for `n >= 1_000`). Same sparse gate as
the stage-5 round 3. Strict exact-score admit → structurally 0 worse.
Entire block sits inside `n + nnz <= FINAL_REFINE_MAX_WORK` (400k), which hard-
bounds worst-case added work.

## Depth sweep

| extra rounds | dev score | delta vs base |
|---|---|---|
| 1 (fixed round 3) | 0.804867 | −0.52 bip |
| 3 | 0.804802 | −1.17 bip |
| 8 | **0.804719** | **−2.00 bip** |
| 16 | 0.804717 | −2.02 bip |

Saturates at 8 — 16 buys 0.02 bip, i.e. the chains terminate on their own well
before the cap. 8 is therefore effectively "unlimited" and is what ships; the
cap is a safety bound, not an operating point.

## Movers (12 / 0, nine families)

chimera_mgw-c16-2031-01, chimera_selby-c16-01, chimera_selby-c16-02,
chp_partload, crudeoil_lee1_07, crudeoil_pooling_ct3, gasprod_sarawak81,
methanol200 (0.7250 → 0.7053, the deepest single converter), powerflow0118p,
powerflow0300p, procurement1large, transswitch0300p.
Buckets: 1k_10k 0.8426 → 0.8425, gt_10k 0.7144 → 0.7139, lt_1k unchanged.

## Failed variant: the material-gain threshold (recorded so it is not retried)

Requiring each round to pay >= 0.2% relative gain (instead of any strict gain)
was tried to cut the added time. It held the score (0.804732, −1.87 bip) and cut
worst-case cost to ~+4%, BUT collapsed breadth from 12 movers to 5, with the
remaining delta dominated by a single row (methanol200). That is the exact shape
of `d5f51fb` — local −3.39 bip on 4 movers, hidden **0.00%, rejected**. Breadth
was judged worth the extra time; the unthresholded loop ships. Do not re-add the
threshold to buy time without first pricing the breadth loss.

## Timing

Probe timing on this box drifts ~23% across a session (measured: two identical
baseline runs of the SAME binary gave worst 1.132 s and 1.578 s; a third pair of
interleaved A/B runs moved 0.852 s → 1.048 s on `crudeoil_lee4_09`). Full-corpus
probe numbers are therefore NOT comparable across time — the 300 back-to-back
rows contend with each other and with thermal state.

The only trustworthy protocol found: interleave base/candidate builds and pair
ADJACENT measurements, `SSI_PROBE_ONLY` + `SSI_PROBE_REPEAT=5` (min of 5).
Measured that way, this package costs **~+12%** on the critical rows
(`crudeoil_lee4_09` 0.929 → 1.043 s isolated). Worst local ~1.05 s, comparable
to the 1.019 s the module header records for a revision known to have passed and
well under the 1.358 s base of 0060, which also passed.

Also worth recording: a row's full-corpus probe time can be ~4x its isolated
time purely from contention (`pooling_rt2tp` n=118: 0.99 s in-corpus vs 0.255 s
isolated, byte-identical in both). Do not price a gate off in-corpus times.

## Next

- The chain now self-terminates, so further depth is closed. The remaining
  named lead is applying the same "the terminal chain is shallower than the
  mid-pipeline chain" audit to the OTHER terminal mechs.
- 76 dev rows are still tied at exactly 1.0000 (54 lt_1k / 17 1k_10k / 5 gt_10k)
  and remain the largest single prize (~17 bip if all moved 1%). The 5 gt_10k
  ties are cheap (0.23-0.46 s) and carry 4.4x leverage each, but are excluded
  from every terminal local-search gate by `rgreedy::MAX_N = 12_000`, whose
  dense n²/8 bitset makes raising it a real memory/cap risk. Unresolved.

---

## OUTCOME: iter111 (8 rounds, both gate arms) FAILED the hidden cap

Submitted as `c617a8eb` on tip `7257386` → **failed** (not rejected: a gate, not
a score miss). The CLI exposes no failure reason; diagnosis below is structural.

**Cause: the `n >= 10_000` arm of the gate.** The loop inherited the stage-5
round-3 gate verbatim, including
`(n >= 10_000 && nnz <= 80_000 && best_flops < amd_flops)`. That put extra
chained `subtree_refine` work on gt_10k rows, which is exactly where the
cap-critical rows live. Measured +12% on `crudeoil_lee4_09` / `arki0016` /
`chimera_rfr-02` against an interleaved parent. The parent passes, so the hidden
margin is under 12% on some row.

Note the asymmetry that made this a trap: the three cap-critical dev rows
(`crudeoil_lee4_09` 118k, `crudeoil_lee4_10` 138k, `ringpack_30_2` 139k) all sit
at `n + nnz >= 117k` and **none of them convert**. The extra rounds bought
nothing there and cost the whole timing margin.

## Option A: sub-10k only (3 rounds) — TIME-SAFE, SCORE-THIN

Gate narrowed to `n < 10_000 && nnz <= 100_000 && n + nnz <= FINAL_REFINE_MAX_WORK`,
depth cut 8 → 3. Every `n >= 10_000` row is then bit-identical to the parent by
construction.

| | score | lt_1k | 1k_10k | gt_10k |
|---|---|---|---|---|
| parent `7257386` | 0.804919 | 0.8880 | 0.8426 | 0.7144 |
| Option A | 0.804905 | 0.8880 | 0.8425 | **0.7144** (identical) |

**−0.14 bip, 7 movers / 0 worse.** Timing, interleaved parent/candidate,
`SSI_PROBE_REPEAT=5`, adjacent pairs (two pairs):

| row | pair 1 | pair 2 |
|---|---|---|
| `arki0016` | +2.5% | −4.3% |
| `chimera_rfr-02` | +1.9% | −0.9% |
| `crudeoil_lee4_09` | +0.4% | −2.2% |
| `crudeoil_lee4_10` | +3.8% | −2.4% |
| `faclay75` | +4.9% | −0.6% |

Mean ≈ +0.1%: **time-neutral within noise**, all rows inside ±5%. This confirms
the diagnosis — removing the gt_10k arm removes the entire +12%.

## Conclusion: this family is CLOSED

The gain and the cap risk came from the SAME rows. Sub-10k-only is time-neutral
but yields −0.14 bip on 7 movers, far short of the ~−0.80 bip local needed for a
1-bip hidden move. Option B (1 extra round, same gate) is strictly weaker: under
an identical gate with strict-improvement admission, 1 round's accepted set is a
subset of 3 rounds', so it cannot exceed −0.14 bip. Not measured for that reason.

**Do not deepen FINAL_REFINE again.** The transferable finding is the audit that
produced it — compare each terminal mech against its mid-pipeline counterpart and
look for unjustified shallowness — but the next application must pick a mech
whose converting rows are NOT the cap-critical ones. Check that overlap FIRST,
before measuring score.
