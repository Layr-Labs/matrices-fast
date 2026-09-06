# 0070: Terminal core relabels on df6e3f0

Base df6e3f0 / submission 5bc73142, hidden 0.861619. Retain the complete
0069 independent-terminal-candidate mechanism and merge the leader's
conditional second core refinement round and 8M completion allowance.
There are no new relabel tickets or broader eligibility gates.

Fresh full trusted sandboxed comparison on all 300 development matrices:
score 0.832566 -> 0.832277; fill 0.938931 -> 0.938888.
Two wins, zero losses, 298 exact ties:

- rsyn0830m04m: 193427 -> 181793 flops (-6.0147%).
- rsyn0820m04m: 177745 -> 167666 flops (-5.6705%).

Buckets: lt_1k 0.889764 unchanged; 1k_10k 0.866260 -> 0.865295;
gt_10k 0.764398 unchanged. All 62 active tests pass (18 diagnostic probes
ignored). All 40 generated sandboxed stress calls pass bijection,
determinism, 4 GiB address-space caps, and 2 s deadlines. Slowest observed
whole-process call: control 0.334633 s, candidate 0.336953 s.

The live benchmark was rechecked after local validation and still named
df6e3f0. Official result pending; no promotion inferred from local scores.
Prior 798e4ffb on a942ebd is tracked separately and was still validating.
