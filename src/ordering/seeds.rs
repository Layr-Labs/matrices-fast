//! Whole-graph candidate portfolio, in stable acceptance order.
use super::{graph_orderers::*, policy::*, *};

pub(super) fn dense_count(degrees: &[usize], alpha: f64) -> usize {
    let n = degrees.len();
    let threshold = if alpha < 0.0 {
        n.saturating_sub(2)
    } else {
        (alpha * (n as f64).sqrt()) as usize
    };
    let threshold = threshold.max(16).min(n);
    degrees.iter().filter(|&&d| d > threshold).count()
}

pub(super) fn degrees(n: usize, cp: &[usize], ri: &[usize]) -> Vec<usize> {
    (0..n)
        .map(|v| ri[cp[v]..cp[v + 1]].iter().filter(|&&u| u != v).count())
        .collect()
}

pub(super) fn refine(
    pattern: &Pattern,
    core: &feral_ordering_core::CscPattern<'_>,
    scoring_pat: &ScoringPattern,
    best: &mut Candidate,
) {
    let n = pattern.n;
    let nnz = pattern.nnz();
    let degrees = degrees(n, &pattern.col_ptr, &pattern.row_idx);
    let traversal = std::sync::OnceLock::new();
    let use_residuals = n >= 1_000
        && pattern
            .col_ptr
            .windows(2)
            .filter(|p| p[1] - p[0] <= 3)
            .count()
            * 4
            >= n;
    let restarts = relabel_restarts(RELABEL_BUDGET, RELABEL_MAX_RESTARTS, nnz);
    let relabelled: Vec<std::sync::OnceLock<(Vec<usize>, Vec<i32>, Vec<i32>)>> =
        (0..restarts).map(|_| std::sync::OnceLock::new()).collect();
    let prepare = |r: usize| {
        relabelled[r].get_or_init(|| {
            let q = relabel(n, r as u64 + 1);
            let b = permute_pattern(scoring_pat, &q);
            let cp = b.col_ptr.iter().map(|&x| x as i32).collect();
            let ri = b.row_idx.iter().map(|&x| x as i32).collect();
            (q, cp, ri)
        })
    };
    // Equal costs keep the first generator in the fixed schedule.
    type Generator<'a> =
        Box<dyn Fn() -> Result<Vec<i32>, feral_ordering_core::OrderingError> + Sync + 'a>;
    let mut tasks: Vec<Generator<'_>> = Vec::new();
    // Threshold sets are nested: equal counts mean exactly the same deferred vertices.
    let mut amd_seen = vec![(true, dense_count(&degrees, 10.0))];
    let mut amf_seen = Vec::new();
    let mut amd = |aggressive, dense_alpha| -> Option<Generator<'_>> {
        let key = (aggressive, dense_count(&degrees, dense_alpha));
        if amd_seen.contains(&key) {
            return None;
        }
        amd_seen.push(key);
        Some(Box::new(move || {
            quotient::order(core, dense_alpha, aggressive, false, quotient::WORK)
        }))
    };
    let mut amf = |dense_alpha| -> Option<Generator<'_>> {
        let key = dense_count(&degrees, dense_alpha);
        if amf_seen.contains(&key) {
            return None;
        }
        amf_seen.push(key);
        Some(Box::new(move || {
            quotient::order(core, dense_alpha, true, true, quotient::WORK)
        }))
    };
    let metis = |opts| -> Generator<'_> {
        Box::new(move || feral_metis::metis_order_full(core, &opts).map(|(p, ..)| p))
    };

    if n < AMF_MAX_N && nnz < AMF_MAX_NNZ {
        tasks.extend(amf(5.0));
    }

    if n < MEDIUM_MAX_N && nnz < MEDIUM_MAX_NNZ {
        for alpha in [5.0, 2.0] {
            tasks.extend(amd(true, alpha));
        }
        if nnz < SWEEP_EXTRA_MAX_NNZ {
            for alpha in [1.0, 16.0] {
                tasks.extend(amd(true, alpha));
            }
        }
    }

    if n < AMF_MAX_N && nnz < 130_000 {
        let alphas: &[f64] = if n < 10_000 {
            &[1.0, 16.0, -1.0]
        } else {
            &[-1.0]
        };
        for &alpha in alphas {
            tasks.extend(amf(alpha));
        }
    } else if n < AMF_MAX_N && nnz >= 400_000 && nnz < AMF_MAX_NNZ {
        tasks.extend(amf(-1.0));
    }

    if n < ROBUST_MAX_N && nnz < ROBUST_MAX_NNZ {
        for alpha in [10.0, 5.0, 2.0, -1.0] {
            tasks.extend(amd(false, alpha));
        }
        tasks.extend(amd(true, -1.0));
    }

    if n < RCM_MAX_N && nnz < RCM_MAX_NNZ {
        let traversal = &traversal;
        tasks.push(Box::new(move || {
            Ok(rcm_order(traversal.get_or_init(|| Graph::new(pattern))))
        }));
    }

    if !use_residuals && n < ND_MAX_N && nnz < ND_MAX_NNZ {
        let traversal = &traversal;
        tasks.push(Box::new(move || {
            Ok(nd_order(traversal.get_or_init(|| Graph::new(pattern))))
        }));
    }

    if !use_residuals && n < NDFM_MAX_N && nnz < NDFM_MAX_NNZ {
        let traversal = &traversal;
        tasks.push(Box::new(move || {
            Ok(ndfm_order(traversal.get_or_init(|| Graph::new(pattern))))
        }));
    }

    if n < MINFILL_MAX_N && nnz < MINFILL_MAX_NNZ {
        tasks.push(Box::new(move || Ok(minfill_order(pattern))));
    }

    if !use_residuals && n < METIS_MAX_N && nnz < METIS_MAX_NNZ {
        tasks.push(metis(feral_metis::MetisOptions::default()));
    }

    if !use_residuals && n < METIS_TUNED_MAX_N && nnz < METIS_TUNED_MAX_NNZ {
        let metis_tuned = feral_metis::MetisOptions {
            niparts: 16,
            fm_passes: 20,
            ..Default::default()
        };
        tasks.push(metis(metis_tuned));
    }

    if !use_residuals && n < METIS_HITRIAL_MAX_N && nnz < METIS_HITRIAL_MAX_NNZ {
        let metis_hitrial = feral_metis::MetisOptions {
            niparts: 32,
            fm_passes: 30,
            ..Default::default()
        };
        tasks.push(metis(metis_hitrial));
    }

    if !use_residuals && n < KAHIP_MAX_N && nnz < KAHIP_MAX_NNZ {
        tasks.push(Box::new(move || feral_kahip::kahip_order(&core)));
    }

    if !use_residuals && n < METIS_VAR_MAX_N && nnz < METIS_VAR_MAX_NNZ {
        for imb in [0.05f64, 0.10] {
            let opts = feral_metis::MetisOptions {
                max_imbalance: imb,
                ..Default::default()
            };
            tasks.push(metis(opts));
        }
        for sw in [100u32, 400] {
            let opts = feral_metis::MetisOptions {
                nd_to_amd_switch: sw,
                ..Default::default()
            };
            tasks.push(metis(opts));
        }
        let opts_seed = feral_metis::MetisOptions {
            seed: 21,
            ..Default::default()
        };
        tasks.push(metis(opts_seed));
    }

    if n < KAHIP_MULTI_MAX_N && nnz < KAHIP_MULTI_MAX_NNZ {
        let kahip_seed2 = feral_kahip::KahipOptions {
            seed: 2,
            ..Default::default()
        };
        tasks.push(Box::new(move || {
            feral_kahip::kahip_order_full(&core, &kahip_seed2).map(|(p, _, _)| p)
        }));

        let kahip_eco = feral_kahip::KahipOptions {
            mode: feral_kahip::KahipMode::Eco,
            ..Default::default()
        };
        tasks.push(Box::new(move || {
            feral_kahip::kahip_order_full(&core, &kahip_eco).map(|(p, _, _)| p)
        }));
    }

    for r in 0..restarts {
        let prepare = &prepare;
        tasks.push(Box::new(move || {
            let (q, bcp, bri) = prepare(r);
            let bcore = feral_ordering_core::CscPattern::new(n, &bcp, &bri)
                .ok_or(feral_ordering_core::OrderingError::MalformedInput)?;
            let pb = quotient::order(&bcore, 10.0, true, false, quotient::WORK)?;
            // Compose back: `q[k]` is the original vertex that B numbers `k`.
            Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
        }));
    }

    if nnz <= RELABEL_AMF_MAX_NNZ {
        for r in 0..restarts {
            let prepare = &prepare;
            tasks.push(Box::new(move || {
                let (q, bcp, bri) = prepare(r);
                let bcore = feral_ordering_core::CscPattern::new(n, &bcp, &bri)
                    .ok_or(feral_ordering_core::OrderingError::MalformedInput)?;
                let pb = quotient::order(&bcore, 5.0, true, true, quotient::WORK)?;
                // Compose back: `q[k]` is the original vertex that B numbers `k`.
                Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
            }));
        }
    }
    let threads = if (n + nnz).saturating_mul(tasks.len()) >= 500_000 {
        4
    } else {
        1
    };
    let results = parallel::map(&tasks, threads, |_, produce, _| {
        let order = std::panic::catch_unwind(std::panic::AssertUnwindSafe(produce))
            .ok()?
            .ok()?;
        Some(order.into_iter().map(|v| v as usize).collect())
    });
    let _ = accept_orders(
        scoring_pat,
        results.into_iter().flatten(),
        &mut best.perm,
        &mut best.cost,
    );
}

/// Deterministic 64-bit mixer (SplitMix64). Used only to derive relabelings from
/// a fixed seed, so every run produces the identical sequence — the determinism
/// gate requires the two `order()` runs to agree byte-for-byte.
pub(super) fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A relabeling of `0..n` derived from a fixed seed (Fisher-Yates over
/// SplitMix64). Pure function of `(n, seed)` — no wall-clock, no entropy.
pub(super) fn relabel(n: usize, seed: u64) -> Vec<usize> {
    let mut q: Vec<usize> = (0..n).collect();
    let mut s = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x1234_5678_9ABC_DEF0);
    for i in (1..n).rev() {
        let j = (splitmix64(&mut s) % (i as u64 + 1)) as usize;
        q.swap(i, j);
    }
    q
}

pub(super) fn relabel_restarts(budget: usize, cap: usize, nnz: usize) -> usize {
    if nnz == 0 {
        return 0;
    }
    (budget / nnz).min(cap)
}
