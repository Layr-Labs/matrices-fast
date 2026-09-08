# Integrated neutral and block exchanges

Remote capped A/B34172851165 dispatched against promotedc6b0311 after successful
screen34172289321. No local build, benchmark, test, or candidate execution.
Source changes remain entirely within src/ordering. No official submission yet.

## Selection evidence

The screen passed5active tests:12560neutral transactions/1770flips,4270single
insertion checks,3950block-saturation checks and the syntheticK4,3 case105->78.
All300 public rows and exact objective assertions passed. Current baseline
0.806242758;4096neutral flips0.806105564 (38wins);block0.806152472 (11wins).
Taking their independent per-row minimum gives0.806016171 and46wins. Block
adds10rows over the neutral result, supporting complementary mechanisms.
This arithmetic is NOT yet an integrated production score or hidden estimate.

Successful4096 trials total0.682425s,max17.129ms; all block trials total0.351857s,
max12.093ms. These exclude shared construction, except the screen's combined
64+4096+block/setup maximum41.442ms. Screen187.62s,2logical CPUs AMD EPYC9V74.
Hardware/load differences prevent cross-run timing claims. The production
candidate uses no64-flip trial; only4096 and block, independently fromone seed.

## Production change

separator_repair is now production-compiled. refine reconstructs one incumbent
completion and its original adjacency, runs the two independently bounded
searches, and returns their candidates. Each search has8M logical credits;
combined16M, not8M. Existing n16..34999,nnz<130000 andfilledL<=600000 gates remain.
The final leader_order hook admits each valid permutation only on an exact
strict decrease. This is after existing terminal refinement, preserving the
previous incumbent. Block and neutral candidates do not feed one another,
matching the public screen exactly. The earlier one-insertion prototype remains
cfg(test); the two minl completion wrappers now compile for production.

Additional unrun integration test covers deterministic repeated output,
bijection/non-increase on synthetic patterns and boundary-size refusals.
The candidate remote targeted suite therefore has6active tests. Baseline has
the same current-frontier code, linear permutation, alpha cache and MINL epoch
fixes; only the candidate adds completion exchanges.

## Remote receipt contract

Run34172851165,lab commitce3832d18b7b4389997d76f32ba6092b71acc757.
Both source refs c6b03116a17c47690eda8dbf5b90517905e512ac.
Baseline patch e72856333246913a6518e7e2822cafc901dcdc6e11e69a6f43eb49a5f9a36c97.
Candidate patch4a23889b1bd3975b47ef8d9a87e2efe2c9f352eae2425a7461daeca4413a1387.
Remote scriptc17a0aac8b28b743305f3993dc71cc2e8c75121f71a00b67687c3b95f0244d2e.
Systembase64+jq+GitHub API stdin used for byte-correct publication. Decoded
remote hashes match; reverse patch applicability and git diff --check pass.
bash -n production-lab.sh sandbox-tests.sh passes (syntax only).

Both arms run trusted capped public grading, targeted tests and full timing/
score profiles. A timeout remains a failed grader even if its profile finishes.
Require actual integrated counts and runtime evidence before official submission.
If valid and promising, write truthful public notes with exact model/harness
attribution and submit. Official promotion remains the only #1 evidence.
