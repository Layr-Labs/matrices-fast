//! Test-only census of exact-distinct independent sets and core passes.
use super::*;
use crate::ordering::{scoring_ws, sorted_permutation, terminal_polish};
use std::time::Instant;

struct Core { il: IndepLift, pattern: ScoringPattern, cp: Vec<i32>, ri: Vec<i32>, amd: u64 }

fn aggregate(s: &[f64;3], c: &[usize;3]) -> f64 {
    [0.3,0.3,0.4].into_iter().enumerate()
        .map(|(i,w)| w*(s[i]/c[i] as f64).exp()).sum()
}

#[test]
#[ignore]
fn probe_independent_candidates() {
    let path = std::env::var("SSI_CAMPAIGN_SEED_CACHE").unwrap();
    let mut cache = std::collections::BTreeMap::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let (name, values) = line.split_once('\t').unwrap();
        cache.insert(name.to_owned(), values.split(',')
            .map(|s| s.parse::<usize>().unwrap()).collect::<Vec<_>>());
    }
    let mut sums = [[0.0;3];10];
    let mut counts = [0;3];
    let mut wins = [[0;3];8];
    let mut times = [0.0f64;8];
    let mut maxima = [0.0f64;8];
    let mut tail_cache = String::new();
    for (name, pat) in crate::corpus::corpus() {
        let original = cache.remove(&name).unwrap();
        let sp = ScoringPattern { n:pat.n,col_ptr:pat.col_ptr.clone(),row_idx:pat.row_idx.clone() };
        let ws = std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder = std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score = |p:&[usize]| ws.borrow_mut().flops(&sp,p);
        let reference = score(&original);
        let tail = terminal_polish::refine(&pat,&original,&score,
            |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()).unwrap_or(original);
        let incumbent = score(&tail);
        tail_cache.push_str(&name);tail_cache.push('\t');
        for (i,v) in tail.iter().enumerate() {
            if i!=0 { tail_cache.push(','); } tail_cache.push_str(&v.to_string());
        }
        tail_cache.push('\n');
        let cp:Vec<i32> = pat.col_ptr.iter().map(|&v|v as i32).collect();
        let ri:Vec<i32> = pat.row_idx.iter().map(|&v|v as i32).collect();
        let raw = feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize> = run_pass(&raw,Pass::Amd).unwrap().into_iter().map(|v|v as usize).collect();
        let baseline = score(&amd);
        let b = if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};
        counts[b]+=1;
        sums[0][b]+=(reference as f64/baseline as f64).ln();
        sums[1][b]+=(incumbent as f64/baseline as f64).ln();
        let mut row = [incumbent;8];
        let mut costs = [0.0f64;8];
        if pat.n>=32 &&pat.n<=18_000 &&pat.nnz()<=80_000 {
            let t = Instant::now();
            let first = greedy_independent_set(&sp,usize::MAX);
            let mut sets = vec![first.clone()];
            let dense = pat.nnz()>=12*pat.n;
            let caps:&[usize] = if dense {&[20,15,9,7,5,3]} else {&[15,9,5,3]};
            for &cap in caps { sets.push(greedy_independent_set(&sp,cap)); }
            for cap in [usize::MAX,15,9,5,16,3] {
                sets.push(greedy_independent_set_excluding(&sp,cap,&first));
            }
            let max_pairs = 8_000_000u64.saturating_sub(5*pat.nnz() as u64)/9;
            let mut distinct = Vec::new();
            let mut cores = Vec::new();
            for mut set in sets {
                budget_trim(&sp,&mut set,max_pairs);
                if distinct.contains(&set) { continue; }
                distinct.push(set.clone());
                let pairs = predicted_pairs(&sp,&set);
                if pat.nnz() as u64+2*pairs>250_000 { continue; }
                let Some(il) = lift(&sp,&set,75_000) else {continue};
                if il.core_n()<2 ||il.core_nnz()>150_000 {continue;}
                let pattern = ScoringPattern {n:il.core_n(),col_ptr:il.core_col_ptr.clone(),row_idx:il.core_row_idx.clone()};
                let cp:Vec<i32> = pattern.col_ptr.iter().map(|&v|v as i32).collect();
                let ri:Vec<i32> = pattern.row_idx.iter().map(|&v|v as i32).collect();
                let core = feral_ordering_core::CscPattern::new(pattern.n,&cp,&ri).unwrap();
                let p:Vec<usize> = run_pass(&core,Pass::Amd).unwrap().into_iter().map(|v|v as usize).collect();
                let f = il.prefix_flops+crate::ordering::flops_of(&pattern,&p);
                let candidate = splice(&il,&p);
                assert_eq!(score(&candidate),f);
                for r in &mut row { *r=(*r).min(f); }
                cores.push(Core{il,pattern,cp,ri,amd:f});
            }
            let setup = t.elapsed().as_secs_f64(); costs.fill(setup);
            let mut ranked:Vec<usize> = (0..cores.len()).collect();
            ranked.sort_by_key(|&i|(cores[i].amd,i));
            let best_amd = ranked.first().map(|&i|cores[i].amd).unwrap_or(u64::MAX);
            for (rank,&i) in ranked.iter().enumerate() {
                let lc = &cores[i];
                if lc.amd.saturating_mul(2)>best_amd.saturating_mul(3) {continue;}
                let core = feral_ordering_core::CscPattern::new(lc.pattern.n,&lc.cp,&lc.ri).unwrap();
                for pass_index in 0..3 {
                    let pass = match pass_index {0=>Pass::Metis,
                        1=>Pass::Metric(crate::ordering::custom_metrics::ScoreVariant::DegP125),
                        _=>Pass::Metric(crate::ordering::custom_metrics::ScoreVariant::AmindNorm)};
                    let t = Instant::now();
                    let Some(p) = run_pass(&core,pass) else {continue};
                    let p:Vec<usize> = p.into_iter().map(|v|v as usize).collect();
                    let f = lc.il.prefix_flops+crate::ordering::flops_of(&lc.pattern,&p);
                    let candidate = splice(&lc.il,&p);
                    assert!(crate::ordering::is_bijection(&candidate,pat.n));
                    assert_eq!(score(&candidate),f);
                    let elapsed = t.elapsed().as_secs_f64();
                    let mut polished = f;
                    let mut polish_seconds = 0.0;
                    if f<incumbent {
                        let t = Instant::now();
                        if let Some(p) = terminal_polish::refine(&pat,&candidate,&score,
                            |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()) {
                            polished = score(&p);
                        }
                        polish_seconds = t.elapsed().as_secs_f64();
                    }
                    for arm in 0..8 {
                        let metis_limit = [1,2,3,3,3,3,usize::MAX,usize::MAX][arm];
                        let metric_limit = [0,0,0,1,2,3,usize::MAX,usize::MAX][arm];
                        let selected = if pass_index==0 {rank<metis_limit} else {rank<metric_limit};
                        if selected {
                            row[arm] = row[arm].min(if arm==7 {polished} else {f});
                            costs[arm]+=elapsed+if arm==7 {polish_seconds} else {0.0};
                        }
                    }
                    println!("INDEP_CANDIDATE\t{name}\t{rank}\t{}\t{}\t{}\t{pass_index}\t{}\t{f}\t{polished}\t{elapsed:.6}",
                        lc.il.prefix.len(),lc.pattern.n,lc.pattern.row_idx.len(),lc.amd);
                }
            }
        }
        print!("INDEP_ROW\t{name}\t{}\t{}\t{baseline}\t{reference}\t{incumbent}",pat.n,pat.nnz());
        for arm in 0..8 {
            assert!(row[arm]<=incumbent);
            sums[arm+2][b]+=(row[arm] as f64/baseline as f64).ln();
            wins[arm][b]+=usize::from(row[arm]<incumbent);
            times[arm]+=costs[arm];maxima[arm]=maxima[arm].max(costs[arm]);
            print!("\t{}",row[arm]);
        } println!();
    }
    let base = aggregate(&sums[0],&counts);let tail = aggregate(&sums[1],&counts);
    assert!((base-0.791864560331).abs()<1e-11);
    assert!((tail-0.791821436276).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-91dac999-campaign-seeds.tsv",tail_cache).unwrap();
    println!("INDEP_BASE parent={base:.12} tail={tail:.12}");
    for arm in 0..8 { println!("INDEP_TOTAL arm={arm} score={:.12} wins={:?} seconds={:.6} max={:.6}",
        aggregate(&sums[arm+2],&counts),wins[arm],times[arm],maxima[arm]); }
}

#[test]
#[ignore]
fn probe_retained_independent_metrics() {
    let path = std::env::var("SSI_CAMPAIGN_SEED_CACHE").unwrap();
    let mut cache = std::collections::BTreeMap::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let (name, values) = line.split_once('\t').unwrap();
        cache.insert(name.to_owned(),values.split(',')
            .map(|s|s.parse::<usize>().unwrap()).collect::<Vec<_>>());
    }
    let mut sums = [[0.0;3];2];let mut counts = [0;3];let mut wins = [0;3];
    let mut seconds = 0.0;let mut maximum = 0.0f64;let mut compared = 0;
    for (name,pat) in crate::corpus::corpus() {
        let p = cache.remove(&name).unwrap();
        let sp = ScoringPattern {n:pat.n,col_ptr:pat.col_ptr.clone(),row_idx:pat.row_idx.clone()};
        let ws = std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let score = |p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let old = score(&p);let factor = ws.borrow().nnz_l();
        let cp:Vec<i32> = pat.col_ptr.iter().map(|&v|v as i32).collect();
        let ri:Vec<i32> = pat.row_idx.iter().map(|&v|v as i32).collect();
        let raw = feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize> = run_pass(&raw,Pass::Amd).unwrap().into_iter().map(|v|v as usize).collect();
        let baseline = score(&amd);let mut new = old;
        if pat.n>=32 &&pat.n<=18_000 &&pat.nnz()<=80_000 {
            let retained = run_with_cores(&sp,8_000_000);
            let plain = run(&sp,8_000_000);
            assert_eq!(retained.as_ref().map(|(f,p,_)|(*f,p)),plain.as_ref().map(|(f,p)|(*f,p)),"{name}: original driver");
            compared+=1;
            if factor<=200_000 {
                if let Some((_,_,cores)) = retained {
                    let t = Instant::now();
                    if let Some(candidate) = cores.refine(old,&score) {
                        assert!(crate::ordering::is_bijection(&candidate,pat.n));
                        new = score(&candidate);assert!(new<old);
                    }
                    let elapsed = t.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
                }
            }
        }
        let b = if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};
        counts[b]+=1;wins[b]+=usize::from(new<old);
        sums[0][b]+=(old as f64/baseline as f64).ln();
        sums[1][b]+=(new as f64/baseline as f64).ln();
        println!("RETAINED_ROW\t{name}\t{}\t{}\t{baseline}\t{old}\t{new}\t{factor}",pat.n,pat.nnz());
    }
    let base = aggregate(&sums[0],&counts);
    assert!((base-0.791864560331).abs()<1e-11);
    println!("RETAINED_TOTAL base={base:.12} score={:.12} wins={wins:?} compared={compared} seconds={seconds:.6} max={maximum:.6}",aggregate(&sums[1],&counts));
}
