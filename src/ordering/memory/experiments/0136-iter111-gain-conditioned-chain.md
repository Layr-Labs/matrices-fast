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
