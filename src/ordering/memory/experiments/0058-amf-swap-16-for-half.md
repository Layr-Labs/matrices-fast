# 0058 — Cost-neutral unlabelled AMF swap: 16.0 → 0.5

- **Date:** 2026-09-04
- **Score:** `0.845281` → **`0.844270`** (−0.001011, −10.11 bips); fill `0.944707` → **`0.944357`**
- **Status:** local full-run WIN (0.844270); same three-ticket envelope as promoted H2 (`4245c79`); submitted

## Hypothesis

0057 added α 0.5 and 2.5 as *extra* `consider()` tickets and failed hidden
(`85dcf2a`). The two unique prizes (`crudeoil_pooling_dt2`, `chp_partload`)
do not require a fourth/fifth ticket if α=16 is never the unique 3-alpha
winner. Swapping 16.0 for 0.5 keeps ticket count, envelope, and worst-case
work identical to H2, which already passed hidden validation.

## Probe (`probe_amf_swap_alpha`)

Conservative accounting: if α=16 is the unique 3-alpha winner and 0.5/2.5
cannot match it, count a loss even if some other `order()` candidate tied.

```
swap 16->0.5  unique_wins=2 losses=0
swap 16->2.5  unique_wins=0 losses=0
score base 0.845281
score 16->0.5 0.844348  delta=-0.000933
score 16->2.5 0.845281  delta=0.000000
bucket lt_1k   0.893120 -> 0.5 0.893120 / 2.5 0.893120  n=147
bucket 1k_10k  0.868390 -> 0.5 0.868331 / 2.5 0.868390  n=108
bucket gt_10k  0.792071 -> 0.5 0.789782 / 2.5 0.792071  n=45
```

Movers (16→0.5 only):

| matrix | n | nnz | before | after |
|---|---:|---:|---:|---:|
| `crudeoil_pooling_dt2` | 18742 | 75910 | 0.837535 | **0.735268** |
| `chp_partload` | 5211 | 16740 | 0.816472 | **0.810464** |

α=2.5 is a zero. lt_1k is a bit-identical control.

## What changed

```rust
for da in [1.0f64, 0.5, -1.0] {
```

was `[1.0, 16.0, -1.0]`. Same three calls, same `n < AMF_SWEEP_MAX_N && nnz < 130_000` gate. No subtree budget, no gate widen, no extra ticket.

## Why this is not 0057 / H3 / R5

Those revisions *added work*. This one replaces one AMF pass with another of the same family and nnz cap. H2 already passed hidden with this envelope. 0057's hidden failure is not a reason to refuse a same-count swap.

## Follow-ups

- Do not put 0.5 *and* 16.0 on this envelope (that is 0057).
- Do not swap in 2.5; it has zero unique wins.
- Do not raise the 130k ceiling.

## Links

- Prior: [0057](0057-amf-alpha-half-and-twofive.md), [0006](0006-cycled-amf-amd-multistart.md)
