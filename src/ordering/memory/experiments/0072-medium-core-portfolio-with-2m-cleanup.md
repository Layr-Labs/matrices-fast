# 0072: Medium-only core portfolio with 2M extra cleanup

Base df6e3f0, hidden 0.861619; full fresh dev control 0.832566.

The previous 798e4ffb submission failed the hidden 2 s cap, with no pattern
identity or phase attribution disclosed. Keep the inherited pipeline intact;
limit new search to 1000 <= n < 10000 and nnz <=50000, with the existing core
bounds 8 <= cn <=4000 and core nnz <=30000. Two structural relabelings and
AMF alpha {2.5,10,0.5,5} make eight fixed tickets. Accumulate their exact best
separately. Only a winner against the complete baseline receives one extra
completion cleanup, limited to 2M credits through completion::refine_limited.
The normal completion entry point retains the leader's 8M allowance.

The new extra cleanup ceiling is 60% below the failed submission's 5M ceiling.
This is a reduction in configured extra work, not proof of hidden timing.
No additional core tickets run in the small or large buckets.

Full trusted sandboxed run: 300 cases, 0.832566 -> 0.832286, fill
0.938931 -> 0.938912. Three wins, zero regressions, 297 ties:
- rsyn0830m04m: 193427 -> 186365 (-3.6510%).
- rsyn0820m04m: 177745 -> 172238 (-3.0983%).
- gasprod_sarawak16: 210806 -> 200978 (-4.6621%).
Buckets: 0.889764 unchanged / 0.866260 -> 0.865327 / 0.764398 unchanged.

All 62 tests pass (18 diagnostic probes ignored). Fourteen generated stress
graphs cover full n 32..10000, including the medium bucket boundaries; all 56
sandboxed calls pass bijection, determinism, 4 GiB address-space caps, and 2 s
deadlines. Maximum observed whole-process time: control 0.372741 s, candidate
0.374891 s. The live leaderboard was rechecked and still named df6e3f0.

Controls: two independent 1M cleanups scored 0.832290; a single 2M cleanup
scored 0.832286, unchanged by the medium gate. Unrestricted alpha-group
cleanup scored 0.832162 but was not submitted. Four extra random core starts
had no further public improvement and were not retained. Official pending.
