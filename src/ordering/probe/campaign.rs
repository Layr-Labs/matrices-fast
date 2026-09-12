//! Test-only screens; cached permutations never enter production ordering.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Step { Peo(usize), Watch(i64), Window(usize, usize, usize, i64), Priority(usize, usize), Search(i64, u64), Gate(u64) }

#[test]
#[ignore]
fn probe_selected_completion_priority() {
    let mut reference=std::env::var("SSI_PRIORITY_REFERENCE_CACHE").ok().map(|path| {
        std::fs::read_to_string(path).unwrap().lines().map(|line| {
            let (name,p)=line.split_once('\t').unwrap();
            (name.to_owned(),p.split(',').map(|v|v.parse::<usize>().unwrap()).collect::<Vec<_>>())
        }).collect::<std::collections::BTreeMap<_,_>>()
    });
    let text=std::fs::read_to_string("/tmp/matrices-fast-compact-core-campaign-seeds.tsv").unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let mut sums=[0.0;3];let mut counts=[0;3];let mut wins=0;let mut seconds=0.0;
    let mut maximum=0.0f64;let mut output=String::new();
    for (name,pat) in crate::corpus::corpus() {
        let parent=cache.remove(&name).unwrap();let mut p=parent.clone();
        let sp=scoring_pattern(&pat);let mut ws=scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());
        let mut builder=sorted_permutation::SortedPermutation::new(&sp);
        let initial=ws.flops(&sp,&p);let factor=ws.nnz_l();let mut f=initial;
        let t=Instant::now();
        if pat.n>=6 &&pat.n<=30_000 &&pat.nnz()<=180_000 &&factor<=300_000 &&initial<=20_000_000_000 {
            for step in [Step::Priority(1,2),Step::Peo(2)] {
                apply_step(&pat,&mut p,&mut f,&mut builder,&mut ws,&sp,step,(30_000,180_000,300_000));
            }
        }
        let elapsed=t.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        let full_ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let full_builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let candidate=post_core::refine_priority(&pat,&parent,|p|full_ws.borrow_mut().flops(&sp,p),
            |p|full_builder.borrow_mut().permute(p),||full_ws.borrow().nnz_l()).unwrap_or(parent);
        assert_eq!(p,candidate,"{name}: independent chain vs selected helper");
        if let Some(cache)=reference.as_mut() {assert_eq!(p,cache.remove(&name).unwrap(),
            "{name}: pre-hard-stop recorded permutation");}
        assert!(f<=initial);wins+=usize::from(f<initial);
        let (cp,ri)=core_of(&pat);let core=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
        let reference=ws.flops(&sp,&amd);let b=bucket(pat.n);counts[b]+=1;
        sums[b]+=(f as f64/reference as f64).ln();
        output.push_str(&name);output.push('\t');
        for (i,v) in p.iter().enumerate() {if i!=0 {output.push(',');}output.push_str(&v.to_string());}output.push('\n');
        println!("SELECTED_PRIORITY_ROW\t{name}\t{}\t{}\t{reference}\t{initial}\t{f}\t{factor}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    assert!(cache.is_empty());let result=aggregate(&sums,&counts);
    if let Some(cache)=reference {assert!(cache.is_empty());}
    assert!((result-0.791259803417).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-wide-priority-campaign-seeds.tsv",output).unwrap();
    println!("SELECTED_PRIORITY_TOTAL score={result:.12} wins={wins} seconds={seconds:.6} max={maximum:.6}");
}

#[test]
#[ignore]
fn probe_early32_final_corpus() {
    let read=|path:&str|->std::collections::BTreeMap<String,Vec<usize>> {
        std::fs::read_to_string(path).unwrap().lines().map(|line| {
            let (name,values)=line.split_once('\t').unwrap();
            (name.to_owned(),values.split(',').map(|s|s.parse().unwrap()).collect())
        }).collect()
    };
    let mut cache=read("/tmp/matrices-fast-5a8c5623-campaign-seeds.tsv");
    let mut parent_cache=read("/tmp/matrices-fast-07f0e8a2-campaign-seeds.tsv");
    let mut sums=[0.0;3];let mut counts=[0;3];let mut wins=0;let mut total=0.0;let mut maximum=0.0f64;
    let mut output=String::new();
    for (name,pat) in crate::corpus::corpus() {
        let raw=cache.remove(&name).unwrap();let parent=parent_cache.remove(&name).unwrap();
        let sp=scoring_pattern(&pat);let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let raw_f=score(&raw);let raw_l=ws.borrow().nnz_l();
        let expected=if raw!=parent {
            assert!(raw_f<score(&parent));
            post_core::refine(&pat,&raw,1,200_000,&score,
                |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()).unwrap_or(raw)
        } else {raw};
        let t=Instant::now();let p=order(&pat);let elapsed=t.elapsed().as_secs_f64();
        assert_eq!(p,expected,"{name}: exact completed ordering after early cap and lazy ledger");
        total+=elapsed;maximum=maximum.max(elapsed);
        let f=score(&p);wins+=usize::from(f<raw_f);assert!(f<=raw_f);
        let (cp,ri)=core_of(&pat);let core=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
        let baseline=score(&amd);let b=bucket(pat.n);counts[b]+=1;sums[b]+=(f as f64/baseline as f64).ln();
        output.push_str(&name);output.push('\t');
        for (i,v) in p.iter().enumerate() {if i!=0 {output.push(',');}output.push_str(&v.to_string());}output.push('\n');
        println!("EARLY32_ROW\t{name}\t{}\t{}\t{baseline}\t{raw_f}\t{f}\t{raw_l}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    let result=aggregate(&sums,&counts);assert!((result-0.791635371701).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-early32-campaign-seeds.tsv",output).unwrap();
    println!("EARLY32_TOTAL score={result:.12} wins={wins} seconds={total:.6} max={maximum:.6}");
}

#[test]
#[ignore]
fn probe_funded_kernel_corpus() {
    let path=std::env::var("SSI_CAMPAIGN_SEED_CACHE").unwrap();
    let expected_score=std::env::var("SSI_COMPARE_EXPECTED_SCORE").ok()
        .map(|s|s.parse::<f64>().unwrap()).unwrap_or(0.791703147252);
    let mut cache=std::collections::BTreeMap::new();
    for line in std::fs::read_to_string(path).unwrap().lines() {
        let (name,values)=line.split_once('\t').unwrap();
        cache.insert(name.to_owned(),values.split(',').map(|s|s.parse::<usize>().unwrap()).collect::<Vec<_>>());
    }
    let mut sums=[0.0;3];let mut counts=[0;3];let mut total=0.0;let mut maximum=0.0f64;
    for (name,pat) in crate::corpus::corpus() {
        let expected=cache.remove(&name).unwrap();let t=Instant::now();let p=order(&pat);
        let elapsed=t.elapsed().as_secs_f64();total+=elapsed;maximum=maximum.max(elapsed);
        assert_eq!(p,expected,"{name}: complete ordering after kernel and allowance changes");
        let sp=scoring_pattern(&pat);let mut ws=scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());
        let f=ws.flops(&sp,&p);let (cp,ri)=core_of(&pat);
        let core=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
        let base=ws.flops(&sp,&amd);let b=bucket(pat.n);counts[b]+=1;
        sums[b]+=(f as f64/base as f64).ln();
        println!("FUNDED_ROW\t{name}\t{}\t{}\t{base}\t{f}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    let result=aggregate(&sums,&counts);assert!((result-expected_score).abs()<1e-11);
    println!("FUNDED_TOTAL score={result:.12} seconds={total:.6} max={maximum:.6}");
}

fn apply_step(pat: &Pattern, p: &mut Vec<usize>, f: &mut u64,
    builder: &mut sorted_permutation::SortedPermutation<'_>,
    ws: &mut scoring_ws::ScoreWorkspace, sp: &ScoringPattern, step: Step,
    limits:(usize,usize,usize)) {
    assert_eq!(ws.flops(sp, p), *f);
    if *f > 20_000_000_000 || ws.nnz_l() > limits.2 as u64 { return; }
    match step {
        Step::Gate(_) => {},
        Step::Peo(rounds) => for _ in 0..rounds {
            let before = *f;
            let pp = builder.permute(p);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            if let Some(ps) = peo_extract::candidates_bounded(pat.n,
                &pp.col_ptr, &pp.row_idx, &et.parent, &counts, p,
                limits.0,limits.1,limits.2) {
                for candidate in ps {
                    assert!(is_bijection(&candidate, pat.n));
                    let nf = ws.flops(sp, &candidate);
                    if nf < *f { *p = candidate; *f = nf; }
                }
            }
            if *f == before { break; }
        },
        Step::Watch(budget) => {
            let pp = builder.permute(p);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            if let Some(candidate) = completion::refine_limited(pat.n,
                &pat.col_ptr, &pat.row_idx, &pp.col_ptr, &pp.row_idx,
                &et.parent, &counts, p, budget) {
                assert!(is_bijection(&candidate, pat.n));
                let nf = ws.flops(sp, &candidate);
                if nf < *f { *p = candidate; *f = nf; }
            }
        },
        Step::Window(width, sweeps, stride, budget) => {
            if let Some(candidate) = rgreedy::sparse_span_window_descent(pat.n,
                &pat.col_ptr, &pat.row_idx, p, width, sweeps, stride, budget) {
                assert!(is_bijection(&candidate, pat.n));
                let nf = ws.flops(sp, &candidate);
                assert!(nf <= *f);
                if nf < *f { *p = candidate; *f = nf; }
            }
        },
        Step::Priority(policy, rounds) => for _ in 0..rounds {
            let before = *f;
            let pp = builder.permute(p);
            let et = EliminationTree::from_pattern(&pp);
            let counts = column_counts_gnp(&pp, &et);
            let degrees: Vec<_> = pat.col_ptr.windows(2).map(|a| a[1] - a[0]).collect();
            if let Some(candidate) = peo_extract::priority_candidate_with_limits(pat.n,
                &pp.col_ptr, &pp.row_idx, &et.parent, &counts, p, &degrees, policy,
                limits.0,limits.1,limits.2) {
                assert!(is_bijection(&candidate, pat.n));
                let nf = ws.flops(sp, &candidate);
                assert!(nf <= *f);
                if nf < *f { *p = candidate; *f = nf; }
            }
            if *f == before { break; }
        },
        Step::Search(budget, seed) => {
            if let Some((candidate, _)) = rgreedy::search(pat.n,
                &pat.col_ptr, &pat.row_idx, p, *f, budget, seed) {
                assert!(is_bijection(&candidate, pat.n));
                let nf = ws.flops(sp, &candidate);
                if nf < *f { *p = candidate; *f = nf; }
            }
        },
    }
}

#[test]
#[ignore]
fn probe_campaign() {
    use Step::*;
    let limit=|name:&str,default:usize|std::env::var(name).ok()
        .map(|s|s.parse::<usize>().unwrap()).unwrap_or(default);
    let limits=(limit("SSI_CAMPAIGN_MAX_N",12_000),limit("SSI_CAMPAIGN_MAX_NNZ",200_000),
        limit("SSI_CAMPAIGN_MAX_FACTOR",150_000));
    let first_arms: &[&[Step]] = &[
        &[Peo(2)], &[Peo(6)], &[Watch(1_000_000)], &[Watch(2_000_000)],
        &[Watch(4_000_000)], &[Peo(2), Watch(2_000_000), Peo(2)],
        &[Watch(2_000_000), Peo(2)],
        &[Window(64, 4, 27, 16_000_000), Peo(2)],
        &[Window(10, 4, 3, 16_000_000), Peo(2)],
        &[Window(7, 4, 2, 16_000_000), Peo(2)],
        &[Window(32, 4, 13, 16_000_000), Peo(2)],
        &[Window(48, 4, 1, 16_000_000), Peo(2)],
    ];
    let priority_arms: &[&[Step]] = &[
        &[Priority(0, 2)], &[Priority(1, 2)], &[Priority(2, 2)],
        &[Priority(3, 2)], &[Priority(4, 2)], &[Priority(5, 2)],
        &[Priority(0, 2), Peo(2)], &[Priority(1, 2), Peo(2)],
        &[Priority(2, 2), Peo(2)], &[Priority(3, 2), Peo(2)],
        &[Priority(4, 2), Peo(2)], &[Priority(5, 2), Peo(2)],
    ];
    let extra_priority_arms:&[&[Step]]=&[
        &[Priority(6,2)],&[Priority(7,2)],&[Priority(8,2)],
        &[Priority(9,2)],&[Priority(10,2)],&[Priority(11,2)],
        &[Priority(6,2),Peo(2)],&[Priority(7,2),Peo(2)],&[Priority(8,2),Peo(2)],
        &[Priority(9,2),Peo(2)],&[Priority(10,2),Peo(2)],&[Priority(11,2),Peo(2)],
    ];
    let mode = std::env::var("SSI_CAMPAIGN_MODE").unwrap_or_default();
    let search_arms: &[&[Step]] = &[
        &[Search(5_000_000, 0xd1b5_4a32_d192_ed03)],
        &[Search(10_000_000, 0xd1b5_4a32_d192_ed03)],
        &[Search(20_000_000, 0xd1b5_4a32_d192_ed03)],
        &[Search(40_000_000, 0xd1b5_4a32_d192_ed03)],
        &[Search(5_000_000, 0xa24b_aed4_963e_e407)],
        &[Search(10_000_000, 0xa24b_aed4_963e_e407)],
        &[Search(20_000_000, 0xa24b_aed4_963e_e407)],
        &[Search(40_000_000, 0xa24b_aed4_963e_e407)],
        &[Search(20_000_000, 0xd1b5_4a32_d192_ed03), Peo(2)],
        &[Search(20_000_000, 0xa24b_aed4_963e_e407), Peo(2)],
        &[Search(40_000_000, 0xd1b5_4a32_d192_ed03), Peo(2), Window(32, 4, 13, 16_000_000), Peo(2)],
        &[Search(40_000_000, 0xa24b_aed4_963e_e407), Peo(2), Window(32, 4, 13, 16_000_000), Peo(2)],
    ];
    let composition_arms: &[&[Step]] = &[
        &[Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Peo(2)],
        &[Window(64,4,27,16_000_000), Window(32,4,13,16_000_000), Peo(2)],
        &[Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2)],
        &[Peo(2), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Peo(2), Watch(1_000_000), Peo(2)],
        &[Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2), Watch(1_000_000), Peo(2)],
        &[Peo(2), Watch(1_000_000), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2)],
        &[Window(32,4,13,16_000_000), Peo(2), Window(64,4,27,16_000_000), Peo(2), Window(10,4,3,16_000_000), Peo(2), Window(7,4,2,16_000_000), Peo(2), Watch(1_000_000), Peo(2)],
        &[Priority(1,2), Peo(2), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2), Watch(1_000_000), Peo(2)],
        &[Peo(2), Gate(75_000), Watch(1_000_000), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2)],
        &[Peo(2), Gate(75_000), Watch(1_000_000), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Peo(2)],
        &[Peo(2), Gate(75_000), Watch(1_000_000), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Gate(150_000), Peo(2)],
        &[Peo(2), Gate(75_000), Window(32,4,13,16_000_000), Window(64,4,27,16_000_000), Window(10,4,3,16_000_000), Window(7,4,2,16_000_000), Peo(2), Watch(1_000_000), Peo(2)],
    ];
    let wide_arms: &[&[Step]] = &[
        &[Window(14,4,5,8_000_000)],
        &[Window(14,2,5,8_000_000)],
        &[Window(14,4,1,8_000_000)],
        &[Window(13,4,4,8_000_000)],
        &[Window(11,4,3,8_000_000)],
        &[Window(14,4,5,16_000_000)],
        &[Window(14,4,5,8_000_000),Peo(2)],
        &[Window(14,4,1,8_000_000),Peo(2)],
        &[Window(13,4,4,8_000_000),Peo(2)],
        &[Window(11,4,3,8_000_000),Peo(2)],
        &[Gate(75_000),Window(14,4,5,8_000_000),Peo(2)],
        &[Gate(30_000),Window(14,4,1,8_000_000),Peo(2)],
    ];
    let arms = match mode.as_str() { "priority" => priority_arms,
        "priorityextra"=>extra_priority_arms,
        "wide"=>wide_arms,
        "search" => search_arms, "composition" => composition_arms, _ => first_arms };
    let cache_path = std::env::var("SSI_CAMPAIGN_SEED_CACHE").ok();
    let expected = std::env::var("SSI_CAMPAIGN_EXPECTED_SCORE").ok()
        .map(|s| s.parse::<f64>().unwrap()).unwrap_or(0.791864560331);
    let mut cache = std::collections::BTreeMap::new();
    if let Some(path) = &cache_path {
        if let Ok(contents) = std::fs::read_to_string(path) {
            for line in contents.lines() {
                let (name, values) = line.split_once('\t').unwrap();
                cache.insert(name.to_owned(), values.split(',')
                    .map(|s| s.parse::<usize>().unwrap()).collect::<Vec<_>>());
            }
        }
    }
    let mut output = String::new();
    let mut sums = vec![[0.0; 3]; arms.len() + 2];
    let mut counts = [0; 3];
    let mut wins = vec![[0; 3]; arms.len()];
    let mut seconds = vec![0.0; arms.len()];
    let mut maxima = vec![0.0f64; arms.len()];
    let mut order_seconds = 0.0;
    let mut union_wins = [0; 3];
    for (name, pat) in crate::corpus::corpus() {
        let incumbent = cache.remove(&name).unwrap_or_else(|| {
            let t = Instant::now(); let p = order(&pat);
            order_seconds += t.elapsed().as_secs_f64(); p
        });
        assert!(is_bijection(&incumbent, pat.n));
        output.push_str(&name); output.push('\t');
        for (i, v) in incumbent.iter().enumerate() {
            if i != 0 { output.push(','); }
            output.push_str(&v.to_string());
        }
        output.push('\n');
        let sp = scoring_pattern(&pat);
        let mut builder = sorted_permutation::SortedPermutation::new(&sp);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n, pat.nnz());
        let reference = ws.flops(&sp, &incumbent);
        let factor_nnz = ws.nnz_l();
        let (cp, ri) = core_of(&pat);
        let core = feral_ordering_core::CscPattern::new(pat.n, &cp, &ri).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core).unwrap()
            .into_iter().map(|v| v as usize).collect();
        let baseline = ws.flops(&sp, &amd);
        let b = bucket(pat.n);
        counts[b] += 1;
        sums[0][b] += (reference as f64 / baseline as f64).ln();
        let gate = pat.n >= 6 && pat.n <= limits.0 && pat.nnz() <= limits.1
            && reference <= 20_000_000_000 && factor_nnz <= limits.2 as u64;
        let mut row = vec![reference; arms.len()];
        for (arm, chain) in arms.iter().enumerate() {
            if gate {
                let t = Instant::now(); let mut current = incumbent.clone();
                let mut factor_limit = limits.2 as u64;
                for &step in *chain {
                    if let Gate(limit) = step { factor_limit = limit; continue; }
                    if factor_limit != limits.2 as u64 {
                        assert_eq!(ws.flops(&sp, &current), row[arm]);
                        if ws.nnz_l() > factor_limit { continue; }
                    }
                    apply_step(&pat, &mut current, &mut row[arm],
                        &mut builder, &mut ws, &sp, step,limits);
                }
                let elapsed = t.elapsed().as_secs_f64();
                seconds[arm] += elapsed; maxima[arm] = maxima[arm].max(elapsed);
            }
            wins[arm][b] += usize::from(row[arm] < reference);
            sums[arm + 1][b] += (row[arm] as f64 / baseline as f64).ln();
        }
        let minimum = *row.iter().min().unwrap();
        union_wins[b] += usize::from(minimum < reference);
        sums[arms.len() + 1][b] += (minimum as f64 / baseline as f64).ln();
        print!("CAMPAIGN_ROW\t{name}\t{}\t{}\t{baseline}\t{reference}\t{factor_nnz}", pat.n, pat.nnz());
        for f in row { print!("\t{f}"); } println!();
    }
    let base = aggregate(&sums[0], &counts);
    assert!((base - expected).abs() < 1e-11, "cache/base mismatch: {base}");
    if let Some(path) = &cache_path { std::fs::write(path, output).unwrap(); }
    println!("CAMPAIGN_BASE score={base:.12} order_seconds={order_seconds:.6}");
    for (arm, chain) in arms.iter().enumerate() {
        println!("CAMPAIGN_TOTAL arm={arm} chain={chain:?} score={:.12} wins={:?} seconds={:.6} max={:.6}",
            aggregate(&sums[arm + 1], &counts), wins[arm], seconds[arm], maxima[arm]);
    }
    println!("CAMPAIGN_ORACLE score={:.12} wins={union_wins:?}",
        aggregate(&sums[arms.len() + 1], &counts));
}

#[test]
#[ignore]
fn probe_retained_independent_stress() {
    let cap_mode=std::env::var_os("SSI_COMPARE_EARLY_CAP").is_some();
    let prior_cap=std::env::var_os("SSI_LADDER_CAP");
    let mut fixtures = Vec::new();
    for (n, links) in [(2_048,4), (2_048,20), (2_048,40),
        (8_000,2), (8_000,4), (8_000,8), (8_000,10), (12_000,4)] {
        let mut state = 0x587a_2834_d139_u64 ^ n as u64 ^ links as u64;
        let mut edges = Vec::new();
        for v in 0..n { for _ in 0..links {
            state ^= state << 13; state ^= state >> 7; state ^= state << 17;
            let u = state as usize % n;
            if u != v { edges.push((v.min(u), v.max(u))); }
        } }
        edges.sort_unstable(); edges.dedup();
        fixtures.push((format!("random_n{n}_links{links}"), Pattern::from_edges(n, &edges)));
    }
    for grid in [32usize,64,96,128] {
        let mut edges = Vec::new();
        for y in 0..grid { for x in 0..grid {
            let v = y*grid+x;
            if x+1<grid { edges.push((v,v+1)); }
            if y+1<grid { edges.push((v,v+grid)); }
        } }
        fixtures.push((format!("grid{grid}"), Pattern::from_edges(grid*grid,&edges)));
    }
    fixtures.push(("hub2048".to_owned(), Pattern::from_edges(2_048,
        &(1..2_048).map(|v| (0,v)).collect::<Vec<_>>())));
    for hub in [false,true] {
        fixtures.push((if hub {"hub16000"} else {"path16000"}.to_owned(),
            Pattern::from_edges(16_000,&(1..16_000)
                .map(|v|if hub {(0,v)} else {(v-1,v)}).collect::<Vec<_>>())));
    }
    for (name,pat) in fixtures {
        let sp = scoring_pattern(&pat);
        let mut ws = scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());
        let mut minima = [f64::MAX;2];
        let mut outputs = [Vec::new(),Vec::new()];
        let mut flops = [0;2];
        let mut old_factor = 0;
        for pair in 0..2 { for offset in 0..2 {
            let arm = (pair+offset)%2;
            indep_first::set_extra_enabled(true);
            indep_first::set_search_enabled(true);
            post_core::set_priority_enabled(cap_mode ||arm==1);
            if cap_mode {std::env::set_var("SSI_LADDER_CAP",if arm==0 {"64"} else {"32"});}
            let t = Instant::now(); let p = order(&pat);
            minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
            assert!(is_bijection(&p,pat.n));
            if pair!=0 { assert_eq!(p,outputs[arm],"{name} arm={arm}"); }
            outputs[arm] = p; flops[arm] = ws.flops(&sp,&outputs[arm]);
            if arm==0 { old_factor = ws.nnz_l(); }
        } }
        if !cap_mode {assert!(flops[1]<=flops[0],"{name}");}
        if !cap_mode &&(old_factor>300_000 ||flops[0]>20_000_000_000) {
            assert_eq!(outputs[0],outputs[1],"{name}: closed refinement");
        }
        println!("CAMPAIGN_STRESS\t{name}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}\t{old_factor}",
            pat.n,pat.nnz(),minima[0],minima[1],flops[0],flops[1]);
    }
    indep_first::set_extra_enabled(true);
    indep_first::set_search_enabled(true);
    post_core::set_priority_enabled(true);
    if cap_mode {
        if let Some(value)=prior_cap {std::env::set_var("SSI_LADDER_CAP",value);}
        else {std::env::remove_var("SSI_LADDER_CAP");}
    }
}
