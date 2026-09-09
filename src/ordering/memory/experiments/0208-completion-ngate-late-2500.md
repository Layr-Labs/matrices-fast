# 0208 — Late post-five completion n-gate widen (n≤2500 / nnz≤40k)

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / worst ~**1.138s**)
- **Promote bar:** hidden ≤**0.843073**; local well past ~0.792300 OR ≤ tip_local−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s multi-probe
- **Status:** **MISS / REVERTED** (no submit; no yukon). Late gate restored to `n≤1000 / nnz≤20k`. 647a transplant stack left intact.
- **Model / harness:** Grok / Grok Bot

## Hypothesis

CoS/Scoreboard next family after closed ledger bumps (0205–0207): **completion n-gate / MINL retune** — structural, not another ledger. No Scoreboard concrete brief on disk → invent one.

Early completion (`n≤30k / nnz≤180k`) runs *before* five/rebuild replacements. The late post-five watcher was capped at `n≤1000 / nnz≤20k`, so ~54 matrices in `1k < n ≤ 2500` that five can replace ship without a second completion pass. Widen that **n-gate** to `n≤2500 / nnz≤40k` (exact-admit, credits stay 4M). Stay strictly below the banned mid-band watcher `4k < n ≤ 15k` (f606aae) and leave lee4 crowns (n~10–18k) untouched. Not a ledger bump; 647a transplant stack intact.

## Change (`src/ordering/mod.rs` only; reverted)

```rust
// was: n >= 16 && n <= 1_000 && nnz <= 20_000
if n >= 16 && n <= 2_500 && nnz <= 40_000 {
    // completion::refine_limited(..., 4_000_000) unchanged
}
```

647a stack kept intact throughout (TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8];
sparse AMD-tie; FINAL_FIVE_OPS=128M; CORE_MINFILL_LEDGER=16M with tip gate cn≤4k /
core_nnz≤30k; INDEP_WORK_LEDGER=8M). No MINL ops/LNNZ/ledger change; no TRANSPLANT→2M;
no 488a / ticket / extrarelbl / chimera.

## Results

Uncapped `probe_timing_and_score` (`SSI_ALLOW_UNSANDBOXED_WORKER=1`, REPEAT=1):
`/tmp/probe-timing-0208.log`

| metric | tip 647a claim | 0208 (late n≤2500) |
|---|---:|---:|
| SCORE | 0.792300 | **0.792300** (Δ 0) |
| WORST order() | ~1.138 s | **1.472 s** (lee4_09) |
| lt_1k / 1k_10k / gt_10k | — | 0.8874 / 0.8397 / 0.6854 |

Crown: lee4_09 **1.472s**, nuclear104 **1.464s**, lee4_10 **1.446s**, lee4_06 **1.407s**.
Per-row vs tip-flat 0205 probe: **0 better / 0 worse / 300 identical** (bit-identical ratios).
Newly gated 1k–2500 band: zero movers — five/rebuild replacements either did not fire there or the early watcher + later stages already exhausted the completion gain.

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 | **FAIL** (identical 0.792300) |
| ≥10 better / 0 worse | **FAIL** (0/0) |
| uncapped worst ≤1.14s | **FAIL** (1.472s; abort threshold) |

`yukon run` **skipped** (score null + timing abort). No submit.

## Decision

**REVERT** late completion gate to `n≤1000 / nnz≤20k`. Leave 647a tip intact.
Late post-five n-gate widen is null on this tip — do not deepen to n≤3k / nnz≤60k without new evidence the band has unfinished completion headroom.

## Blockers / next

- Late completion widen 1k→2.5k: score-null (0/0) and box worst >1.14s (lee4 crowns; change does not touch them).
- Do **not** retry mid-band watcher 4k–15k (f606aae).
- Remaining structural family options (if CoS still wants completion/MINL): MINL `MINL_MAX_LNNZ` / upper-n retune (careful of lee4_09 at L≈558k), or early-completion nnz widen (only ~3 rows in 180–250k — below ≥10 bar alone).
