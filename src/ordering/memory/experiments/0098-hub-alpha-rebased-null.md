# 0098 — Hub-scale second heavy alpha on 68ec638 (re-test after rebase)

- **Date:** 2026-09-07
- **Score:** 0.806616 → 0.806616 (all 300 ratios identical)
- **Status:** NEGATIVE, reverted. The 0097 mechanism does not transfer: the
  new crown's medium-exact tickets + sweep extras already harvest the rows
  hub-alpha moved on 017a036.

## Note

Same construction as 0097 (patch re-applied cleanly). On 017a036: −67 bips,
48 movers. On 68ec638: zero. The difference is the tree, not the idea —
68ec638's own additions cover the same 1k_10k rows first. Mechanism value is
fully absorbed; do not retry hub-alpha variants on this crown.

## Follow-ups

- Donor/hub/alpha axes on the heavy arm: closed for this crown.
