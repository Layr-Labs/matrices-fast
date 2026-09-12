//! Exact prefix/residual extraction and a budgeted residual portfolio.
use super::{policy::*, seeds::relabel, *};

type OrderResult = Result<Vec<i32>, feral_ordering_core::OrderingError>;

fn produce_source(produce: impl FnOnce() -> OrderResult) -> Option<Vec<usize>> {
    let order = std::panic::catch_unwind(std::panic::AssertUnwindSafe(produce))
        .ok()?
        .ok()?;
    Some(order.into_iter().map(|v| v as usize).collect())
}

fn relabelled_order(graph: &ScoringPattern, seed: u64, metis: bool) -> OrderResult {
    let q = relabel(graph.n, seed);
    let permuted = permute_pattern(graph, &q);
    let cp: Vec<i32> = permuted.col_ptr.iter().map(|&v| v as i32).collect();
    let ri: Vec<i32> = permuted.row_idx.iter().map(|&v| v as i32).collect();
    let core = feral_ordering_core::CscPattern::new(graph.n, &cp, &ri)
        .ok_or(feral_ordering_core::OrderingError::MalformedInput)?;
    let order = if metis {
        feral_metis::metis_order_full(&core, &feral_metis::MetisOptions::default())?.0
    } else {
        feral_amd::amd_order(&core)?
    };
    Ok(order.into_iter().map(|v| q[v as usize] as i32).collect())
}

fn native_order(
    core: &feral_ordering_core::CscPattern<'_>,
    graph: &ScoringPattern,
    source: usize,
) -> OrderResult {
    match source {
        0 => feral_amd::amd_order(core),
        1..=3 => {
            let (aggressive, dense_alpha) = [(true, 2.0), (false, -1.0), (false, 10.0)][source - 1];
            feral_amd::amd_order_opts(
                core,
                &feral_amd::AmdOptions {
                    aggressive,
                    dense_alpha,
                },
            )
            .map(|(p, ..)| p)
        }
        4 | 5 => feral_amf::amf_order_opts(
            core,
            &feral_amf::AmfOptions {
                dense_alpha: if source == 4 { 5.0 } else { -1.0 },
                ..Default::default()
            },
        )
        .map(|(p, ..)| p),
        6 => feral_metis::metis_order_full(core, &feral_metis::MetisOptions::default())
            .map(|(p, ..)| p),
        7 => feral_metis::metis_order_full(
            core,
            &feral_metis::MetisOptions {
                niparts: 4,
                fm_passes: 20,
                nd_to_amd_switch: 100,
                max_imbalance: 0.10,
                ..Default::default()
            },
        )
        .map(|(p, ..)| p),
        8 => feral_metis::metis_order_full(
            core,
            &feral_metis::MetisOptions {
                niparts: 16,
                fm_passes: 10,
                nd_to_amd_switch: 400,
                max_imbalance: 0.30,
                ..Default::default()
            },
        )
        .map(|(p, ..)| p),
        9 => relabelled_order(graph, 1, true),
        _ => unreachable!(),
    }
}

pub(super) fn refine(
    pattern: &Pattern,
    scoring_pat: &ScoringPattern,
    best: &mut Candidate,
) -> Option<ladder::Core> {
    let n = pattern.n;
    let mut core3 = None;
    if n >= 20 {
        const LADDER_MAX_CAP: usize = 12;
        const LADDER_PAIR_BUDGET: u64 = 3_000_000;
        const CORE_METIS_MAX_N: usize = 60_000;
        const CORE_METIS_MAX_NNZ: usize = 1_200_000;
        let large_core_n = n > LARGE_CORE_N;
        let thresholds: &[usize] = &[3, 6, 12];
        let mut ladder =
            ladder::CoreLadder::new(n, &pattern.col_ptr, &pattern.row_idx, LADDER_MAX_CAP);
        let mut residuals = Vec::with_capacity(thresholds.len());
        for &goal in thresholds {
            ladder.advance(goal, LADDER_PAIR_BUDGET);
            residuals.push(ladder.export());
        }
        let defer_partition: Vec<_> = residuals
            .iter()
            .enumerate()
            .map(|(i, r)| {
                n >= 10_000
                    && r.ids.len() >= 4000
                    && residuals
                        .get(i + 1)
                        .is_some_and(|next| next.ids.len() * 10 <= r.ids.len() * 9)
            })
            .collect();
        drop(ladder);
        let mut prev_core_n = n;

        let mut dense_ledger = ladder::WorkLedger::new(DENSE_GREEDY_BUDGET);
        let mut dense_work = 16_000_000usize;
        let mut core_pass_ledger = ladder::WorkLedger::new(CORE_PASS_BUDGET);
        for (stage, exported) in residuals.into_iter().enumerate() {
            let goal = thresholds[stage];
            let exported = if goal == 3 {
                core3.insert(exported)
            } else {
                &exported
            };
            let k = exported.ids.len();
            if k == 0 || (k as f64) > 0.9 * prev_core_n as f64 {
                continue;
            }
            prev_core_n = k;
            let m = exported.row_idx.len();
            if let Some(q) = linegraph::order(&exported.col_ptr, &exported.row_idx) {
                let mut full = exported.prefix.clone();
                full.extend(q.iter().map(|&v| exported.ids[v]));
                accept_if_cheaper(Some(full), n, scoring_pat, &mut best.perm, &mut best.cost);
                // The recognized residual has a constructive optimum for this
                // fixed prefix. Further heuristics on this same residual cannot
                // improve it; other prefixes and whole-graph stages still run.
                continue;
            }
            let core_sp = ScoringPattern {
                n: k,
                col_ptr: exported.col_ptr.clone(),
                row_idx: exported.row_idx.clone(),
            };

            // Cheap candidate: project the whole-graph incumbent onto the core.
            let mut pos_in_core = vec![usize::MAX; n];
            for (i, &v) in exported.ids.iter().enumerate() {
                pos_in_core[v] = i;
            }
            let mut core_best_perm: Vec<usize> = best
                .perm
                .iter()
                .filter_map(|&v| {
                    let p = pos_in_core[v];
                    (p != usize::MAX).then_some(p)
                })
                .collect();
            let (mut core_best_flops, entries) = cost_and_entries(&core_sp, &core_best_perm);
            if entries == k + m / 2 {
                // A fill-free residual order attains the fixed-prefix floor.
                // Keep its splice, then continue with other peeling thresholds.
                if exported.prefix_flops + core_best_flops < best.cost {
                    let mut full = exported.prefix.clone();
                    full.extend(core_best_perm.iter().map(|&v| exported.ids[v]));
                    best.perm = full;
                    best.cost = exported.prefix_flops + core_best_flops;
                }
                continue;
            }

            let core_cp_i32: Vec<i32> = exported.col_ptr.iter().map(|&x| x as i32).collect();
            let core_ri_i32: Vec<i32> = exported.row_idx.iter().map(|&x| x as i32).collect();
            let Some(core_core) =
                feral_ordering_core::CscPattern::new(k, &core_cp_i32, &core_ri_i32)
            else {
                continue;
            };

            let core_pass_cost = (k as u64) + (m as u64);
            // Providers read only the residual. Charge in stable order, run
            // independently, then accept in that same order on strict gains.
            let mut sources: Vec<_> = (0..10)
                .filter(|&source| {
                    if k + m > 1_000_000 {
                        return false;
                    }
                    if k + m > 400_000 && !matches!(source, 0 | 2 | 4 | 6) {
                        return false;
                    }
                    let eligible = match source {
                        4 | 5 => k < AMF_MAX_N && m < AMF_MAX_NNZ && (source == 4 || !large_core_n),
                        6..=9 => {
                            !defer_partition[stage]
                                && k < CORE_METIS_MAX_N
                                && m < CORE_METIS_MAX_NNZ
                                && (source == 6 || !large_core_n)
                        }
                        _ => true,
                    };
                    eligible && core_pass_ledger.try_charge(core_pass_cost)
                })
                .collect();
            let degrees = seeds::degrees(k, &exported.col_ptr, &exported.row_idx);
            let mut native_seen = Vec::new();
            sources.retain(|&source| {
                let (kind, alpha) = match source {
                    0 => (0, 10.0),
                    1 => (0, 2.0),
                    2 => (1, -1.0),
                    3 => (1, 10.0),
                    4 => (2, 5.0),
                    5 => (2, -1.0),
                    _ => return true,
                };
                let key = (kind, seeds::dense_count(&degrees, alpha));
                if native_seen.contains(&key) {
                    return false;
                }
                native_seen.push(key);
                true
            });
            let split = sources.partition_point(|&source| source < 6);
            let evaluate = |sources: &[usize]| {
                parallel::map(
                    sources,
                    if k + m >= 25_000 { 4 } else { 1 },
                    |_, &source, _| produce_source(|| native_order(&core_core, &core_sp, source)),
                )
            };
            let results = evaluate(&sources[..split]);
            let has_amd =
                sources.first() == Some(&0) && results.first().is_some_and(Option::is_some);
            let first_cost = accept_orders(
                &core_sp,
                results.into_iter().flatten(),
                &mut core_best_perm,
                &mut core_best_flops,
            );
            let amd_flops = first_cost.filter(|_| has_amd);
            if k < 20_000
                || k + m <= 400_000
                || (exported.prefix_flops + core_best_flops).saturating_mul(100)
                    <= best.cost.saturating_mul(99)
            {
                accept_orders(
                    &core_sp,
                    evaluate(&sources[split..]).into_iter().flatten(),
                    &mut core_best_perm,
                    &mut core_best_flops,
                );
            }

            // Relabelled-AMD restarts, budgeted by core size (fewer on large
            // cores); throttled to a single restart once AMD is already
            // clearly behind the best found so far on this core. Never run
            // on a large-original-matrix core.
            let cap_restarts = if k < 4000 { 24usize } else { 4usize };
            let mut core_restarts = if large_core_n {
                0
            } else {
                (1_000_000usize / (k + m + 1)).clamp(1, cap_restarts)
            };
            // The comparison can only reduce two or more restarts to one.
            // Do not construct and score another AMD order when it cannot
            // change the schedule.
            if core_restarts > 1
                && amd_flops.is_some_and(|f| f as f64 > 1.1 * core_best_flops as f64)
            {
                core_restarts = 1;
            }

            let restarts: Vec<_> = (0..core_restarts)
                .take_while(|_| core_pass_ledger.try_charge(core_pass_cost))
                .collect();
            let results = map_indexed(&restarts, |_, &r| {
                produce_source(|| relabelled_order(&core_sp, r as u64 + 1, false))
            });
            let _ = accept_orders(
                &core_sp,
                results.into_iter().flatten(),
                &mut core_best_perm,
                &mut core_best_flops,
            );

            let split_competitive =
                (exported.prefix_flops + core_best_flops) as f64 <= 1.1 * best.cost as f64;

            if k <= 3500 && m < 300_000 && (k <= 1800 || split_competitive) {
                let modes: &[usize] = if k <= 1800 {
                    &[0, 1, 2, 3, 4, 5]
                } else {
                    &[1, 2, 5]
                };
                let mode_cost = ladder::dense_greedy_cost_estimate(k, m);
                for &mode in modes {
                    if !dense_ledger.try_charge(mode_cost) {
                        break;
                    }

                    let perm = ladder::dense_greedy(
                        k,
                        &exported.col_ptr,
                        &exported.row_idx,
                        mode,
                        7,
                        &mut dense_work,
                    );
                    if is_bijection(&perm, k) {
                        let f = flops_of(&core_sp, &perm);
                        if f < core_best_flops {
                            core_best_flops = f;
                            core_best_perm = perm;
                        }
                    }
                }
            }

            // Accept the splice only on a strict decrease of the exact,
            // whole-graph cost.
            if exported.prefix_flops + core_best_flops < best.cost {
                let mut spliced = exported.prefix.clone();
                spliced.extend(core_best_perm.iter().map(|&i| exported.ids[i]));
                best.cost = exported.prefix_flops + core_best_flops;
                best.perm = spliced;
            }
        }
    }
    core3
}
