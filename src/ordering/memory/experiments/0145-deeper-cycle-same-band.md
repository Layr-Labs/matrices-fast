# 0145 — deeper 3-cycle + 4-cycle in the a004c1c band

- **Date:** 2026-09-08
- **Score:** a004c1c 0.804869 → **0.804866** (−0.03 bip). 0.887807 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Follow-up to `a004c1c` REJECT thin.

## Hypothesis

`a004c1c` rejected thin at hidden 0.848875 (timing passed). Do not widen n. 768 3-cycles in 100≤n≤200 nnz≤2k left korcns on the table. Search the same band harder.

## What changed

Same gate. 2048 3-cycles, then a 4-orbit (768), then a second 3-cycle seed (768). One exact admit. Dense insert and SS-winner pair were public no-ops and are not shipped.

## Result

pooling_adhya4tp still 20540. **korcns** (n=156, nnz=712) 6629→**6618**. 1k_10k/gt_10k crown. Worst isolated maxcsp 1.54 s.

## Follow-ups

- On REJECT thin: do not widen n; leftover-window2 / insert / SS-pair are dead on public.
- On FAIL: the extra draws timed out; revert to a004c1c 768.
- On PROMOTE: next increment still n≤1000, not leftover n>1000 / extra etree / nnz>30k LNS / all-n lottery / unconditioned n≤200 3-cycle.
