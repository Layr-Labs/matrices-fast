//! Test-only screens; cached permutations never enter production ordering.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Step { Peo(usize), Lex(usize), Watch(i64), Window(usize, usize, usize, i64), Priority(usize, usize), Search(i64, u64), Gate(u64) }

#[test]
#[ignore]
fn probe_second_policy_extra_peo() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ordering/memory/evidence");
    let mut seeds:std::collections::BTreeMap<String,Vec<usize>>=std::fs::read_to_string(
        "/tmp/matrices-fast-original-two-policy-production-campaign-seeds.tsv").unwrap().lines()
        .map(|line| {let (name,p)=line.split_once('\t').unwrap();
            (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())}).collect();
    let refs:std::collections::BTreeMap<String,u64>=std::fs::read_to_string(root.join("0222-final-screen.tsv"))
        .unwrap().lines().skip(1).map(|line| {let q:Vec<_>=line.split('\t').collect();
            (q[0].to_owned(),q[3].parse().unwrap())}).collect();
    let flags:std::collections::BTreeMap<String,u8>=std::fs::read_to_string(root.join("0222-production-acceptance-flags.tsv"))
        .unwrap().lines().skip(1).map(|line| {let (name,f)=line.split_once('\t').unwrap();
            (name.to_owned(),f.parse().unwrap())}).collect();
    let mut sums=[0.0;3];let mut base=[0.0;3];let mut counts=[0;3];let mut wins=0;
    let mut output=String::new();let mut seconds=0.0;let mut maximum=0.0f64;
    for (name,pat) in crate::corpus::corpus() {
        let seed=seeds.remove(&name).unwrap();let sp=scoring_pattern(&pat);let amd=refs[&name];
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);let initial=score(&seed);
        let start=Instant::now();let p=if flags[&name]&8!=0 {post_core::refine_bounded(&pat,&seed,2,
            50_000,180_000,750_000,&score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l())
            .unwrap_or(seed)} else {seed};
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        assert!(is_bijection(&p,pat.n));let f=score(&p);assert!(f<=initial);wins+=usize::from(f<initial);
        let b=bucket(pat.n);counts[b]+=1;sums[b]+=(f as f64/amd as f64).ln();base[b]+=(initial as f64/amd as f64).ln();
        output.push_str(&format!("{name}\t{}\n",p.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(",")));
        println!("EXTRA_SECOND_PEO_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{f}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    assert!(seeds.is_empty());assert!((aggregate(&base,&counts)-0.790535759745).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-second-policy-extra-peo-campaign-seeds.tsv",output).unwrap();
    println!("EXTRA_SECOND_PEO_TOTAL score={:.12} wins={wins} seconds={seconds:.6} maximum={maximum:.6}",aggregate(&sums,&counts));
}

#[test]
#[ignore]
fn probe_original_structured_mesh_stress() {
    let depth_mode=std::env::var_os("SSI_MESH_PEO_DEPTH_COMPARISON").is_some();
    for variant in 0..4 {for (w,h) in [(128usize,128usize),(96,192)] {
        let n=w*h;let mut edges=Vec::new();let mut state=0x689f_105c_u64^variant as u64^n as u64;
        for y in 0..h {for x in 0..w {
            let v=y*w+x;
            if x+1<w {edges.push((v,v+1));}if y+1<h {edges.push((v,v+w));}
            state^=state<<13;state^=state>>7;state^=state<<17;
            if x+1<w &&y+1<h &&state%4==0 {edges.push((v,v+w+1));}
            if variant==1 {let hub=(y/8*8)*w+x/8*8;if hub!=v {edges.push((hub,v));}}
            if variant>=2 {
                let tile=if variant==2 {8} else {16};
                let u=(y/tile*tile+(state as usize/tile)%tile)*w+x/tile*tile+state as usize%tile;
                if u<n &&u!=v {edges.push((u.min(v),u.max(v)));}
            }
        }}
        edges.sort_unstable();edges.dedup();let pat=Pattern::from_edges(n,&edges);
        let sp=scoring_pattern(&pat);let mut ws=scoring_ws::ScoreWorkspace::new(n,pat.nnz());
        let (cp,ri)=core_of(&pat);let core=feral_ordering_core::CscPattern::new(n,&cp,&ri).unwrap();
        let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
        let af=ws.flops(&sp,&amd);let mut outputs=[Vec::new(),Vec::new()];let mut f=[0;2];
        let mut minima=[f64::MAX;2];let mut flags=[0;2];
        for pair in 0..2 {for offset in 0..2 {
            let arm=(pair+offset)%2;set_original_large_research(Some(depth_mode ||arm==1));set_original_second_research(Some(depth_mode ||arm==1));
            post_core::set_original_second_peo_rounds(Some(if depth_mode &&arm==1 {4} else {2}));
            take_completion_win_flags();let start=Instant::now();let p=order(&pat);
            minima[arm]=minima[arm].min(start.elapsed().as_secs_f64());let flag=take_completion_win_flags();
            assert!(is_bijection(&p,n));let q=ws.flops(&sp,&p);assert!(q<=af);
            if pair!=0 {assert_eq!(p,outputs[arm]);assert_eq!(flag,flags[arm]);}
            outputs[arm]=p;f[arm]=q;flags[arm]=flag;
        }}
        assert!(f[1]<=f[0]);if flags[1]&4==0 {assert_eq!(outputs[0],outputs[1]);}
        assert!(flags[1]&8==0 ||flags[1]&4!=0);
        println!("ORIGINAL_MESH_STRESS\tmesh{w}x{h}_v{variant}\t{n}\t{}\t{af}\t{}\t{}\t{:.6}\t{:.6}\t{}",
            pat.nnz(),f[0],f[1],minima[0],minima[1],flags[1]);
    }}
    set_original_large_research(None);set_original_second_research(None);take_completion_win_flags();
    post_core::set_original_second_peo_rounds(None);
}

#[test]
#[ignore]
fn probe_conditional_original_second_policy() {
    let read=|path:&str|->std::collections::BTreeMap<String,Vec<usize>> {
        std::fs::read_to_string(path).unwrap().lines().map(|line| {
            let (name,p)=line.split_once('\t').unwrap();
            (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
        }).collect()
    };
    let mut seeds=read("/tmp/matrices-fast-direct-original-root-campaign-seeds.tsv");
    let mut parents=read("/tmp/matrices-fast-shared-terminal-core-campaign-seeds.tsv");
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ordering/memory/evidence");
    let refs:std::collections::BTreeMap<String,u64>=std::fs::read_to_string(root.join("0220-final-screen.tsv"))
        .unwrap().lines().skip(1).map(|line| {let q:Vec<_>=line.split('\t').collect();
            (q[0].to_owned(),q[3].parse().unwrap())}).collect();
    let expected:std::collections::BTreeMap<String,u64>=std::fs::read_to_string(root.join("0221-conditional-second-policy-research.tsv"))
        .unwrap().lines().skip(1).map(|line| {let q:Vec<_>=line.split('\t').collect();
            (q[0].to_owned(),q[8].parse().unwrap())}).collect();
    let mut sums=[0.0;3];let mut base=[0.0;3];let mut counts=[0;3];
    let mut output=String::new();let mut seconds=0.0;let mut maximum=0.0f64;let mut wins=0;
    for (name,pat) in crate::corpus::corpus() {
        let seed=seeds.remove(&name).unwrap();let parent=parents.remove(&name).unwrap();
        let sp=scoring_pattern(&pat);let amd=refs[&name];
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let parent_f=score(&parent);let initial=score(&seed);
        let admitted=initial<parent_f &&pat.n>rgreedy::MAX_N &&amd<=post_core::AMD_WORK_LIMIT &&initial<amd;
        let start=Instant::now();let p=if admitted {post_core::refine_original_policy(&pat,&seed,2,
            &score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()).unwrap_or(seed)} else {seed};
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        assert!(is_bijection(&p,pat.n));let f=score(&p);assert!(f<=initial);
        assert_eq!(f,expected.get(&name).copied().unwrap_or(initial),"{name}: smaller-cap second policy");
        wins+=usize::from(f<initial);let b=bucket(pat.n);counts[b]+=1;
        sums[b]+=(f as f64/amd as f64).ln();base[b]+=(initial as f64/amd as f64).ln();
        output.push_str(&format!("{name}\t{}\n",p.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(",")));
        println!("SECOND_ORIGINAL_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{f}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    assert!(seeds.is_empty() &&parents.is_empty());
    assert!((aggregate(&base,&counts)-0.790789084587).abs()<1e-11);
    assert!((aggregate(&sums,&counts)-0.790535759745).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-conditional-original-second-campaign-seeds.tsv",output).unwrap();
    println!("SECOND_ORIGINAL_TOTAL score={:.12} wins={wins} seconds={seconds:.6} maximum={maximum:.6}",aggregate(&sums,&counts));
}

#[test]
#[ignore]
fn probe_selected_original_incidence_bonus() {
    let direct=std::env::var_os("SSI_DIRECT_ORIGINAL_LARGE_SCREEN").is_some();
    let cache_path=if direct {"/tmp/matrices-fast-validated-frontier-core-campaign-seeds.tsv"}
        else {"/tmp/matrices-fast-state-gated-completion-campaign-seeds.tsv"};
    let expected_base=if direct {0.791087437358} else {0.790176632480};
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ordering/memory/evidence");
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=
        std::fs::read_to_string(cache_path)
        .unwrap().lines().map(|line| {
            let (name,p)=line.split_once('\t').unwrap();
            (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
        }).collect();
    let flags:std::collections::BTreeMap<String,u8>=if direct {std::collections::BTreeMap::new()} else {std::fs::read_to_string(root.join("0217-completion-state-flags.tsv"))
        .unwrap().lines().skip(1).map(|line| {let (name,f)=line.split_once('\t').unwrap();
            (name.to_owned(),f.parse().unwrap())}).collect()};
    let refs:std::collections::BTreeMap<String,u64>=std::fs::read_to_string(root.join(if direct {"0219-final-screen.tsv"} else {"0217-final-screen.tsv"}))
        .unwrap().lines().skip(1).map(|line| {let r:Vec<_>=line.split('\t').collect();
            (r[0].to_owned(),r[3].parse().unwrap())}).collect();
    let mut sums=[0.0;3];let mut base=[0.0;3];let mut counts=[0;3];
    let mut wins=0;let mut seconds=0.0;let mut maximum=0.0f64;let mut output=String::new();let mut eligible=0;
    for (name,pat) in crate::corpus::corpus() {
        let seed=cache.remove(&name).unwrap();let sp=scoring_pattern(&pat);let amd=refs[&name];
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);let initial=score(&seed);let fill=ws.borrow().nnz_l();
        let admitted=amd<=post_core::AMD_WORK_LIMIT &&if direct {
            pat.n>rgreedy::MAX_N &&initial<amd
        } else {flags[&name]&1!=0};
        let mut expected=seed.clone();
        if admitted &&pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=180_000 &&fill<=750_000 {
            eligible+=1;let pp=builder.borrow_mut().permute(&seed);let et=EliminationTree::from_pattern(&pp);
            let cc=column_counts_gnp(&pp,&et);
            if let Some(mut candidates)=peo_extract::original_candidates_with_limits(pat.n,
                &pat.col_ptr,&pat.row_idx,&pp.col_ptr,&pp.row_idx,&et.parent,&cc,&seed,
                &[3],50_000,180_000,750_000) {
                let q=candidates.pop().unwrap();assert!(score(&q)<=initial);
                if score(&q)<initial {expected=post_core::refine_bounded(&pat,&q,2,50_000,180_000,750_000,
                    &score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()).unwrap_or(q);}
            }
        }
        let start=Instant::now();let p=if admitted {post_core::refine_original(&pat,&seed,
            &score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()).unwrap_or(seed)} else {seed};
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        assert_eq!(p,expected,"{name}: selected bounded policy and continuation");
        assert!(is_bijection(&p,pat.n));let f=score(&p);assert!(f<=initial);wins+=usize::from(f<initial);
        let b=bucket(pat.n);counts[b]+=1;sums[b]+=(f as f64/amd as f64).ln();base[b]+=(initial as f64/amd as f64).ln();
        output.push_str(&format!("{name}\t{}\n",p.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(",")));
        println!("SELECTED_ORIGINAL_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{fill}\t{f}\t{elapsed:.6}\t{}",
            pat.n,pat.nnz(),if direct {u8::from(admitted)} else {flags[&name]});
    }
    assert!(cache.is_empty());assert!((aggregate(&base,&counts)-expected_base).abs()<1e-11);
    if !direct {assert!((aggregate(&sums,&counts)-0.790098120936).abs()<1e-11);}
    std::fs::write(if direct {"/tmp/matrices-fast-direct-original-bonus-campaign-seeds.tsv"}
        else {"/tmp/matrices-fast-selected-original-bonus-campaign-seeds.tsv"},output).unwrap();
    println!("SELECTED_ORIGINAL_TOTAL score={:.12} wins={wins} eligible={eligible} seconds={seconds:.6} maximum={maximum:.6}",aggregate(&sums,&counts));
}

#[test]
#[ignore]
fn probe_completion_state_gate_cost() {
    let text=std::fs::read_to_string("/tmp/matrices-fast-state-gated-completion-campaign-seeds.tsv").unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let mut totals=[0.0;2];let mut maxima=[0.0f64;2];let mut recon=[0u64;2];
    let mut lex_calls=[0u64;2];let mut differences=0;let mut eligible=0;
    for (name,pat) in crate::corpus::corpus() {
        let seed=cache.remove(&name).unwrap();let sp=scoring_pattern(&pat);
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let initial=score(&seed);let fill=ws.borrow().nnz_l();
        let mut minima=[f64::MAX;2];let mut outputs=[Vec::new(),Vec::new()];
        let mut calls=[0u64;2];let mut lex=[0u64;2];
        if pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=1_300_000 &&fill<=1_000_000
            &&initial<=20_000_000_000 {
            eligible+=1;
            for pair in 0..2 {for offset in 0..2 {
                let arm=(pair+offset)%2;post_core::set_priority_post_win_only(Some(arm==1));
                let mut p=seed.clone();peo_extract::prof::take();
                let start=Instant::now();
                let first=post_core::refine_priority(&pat,&p,&score,
                    |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l());
                let changed=first.is_some();if let Some(q)=first {p=q;}
                let do_lex=arm==0 ||changed;
                if do_lex {if let Some(q)=post_core::refine_lex(&pat,&p,&score,
                    |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()) {p=q;}}
                minima[arm]=minima[arm].min(start.elapsed().as_secs_f64());
                let count=peo_extract::prof::take().2;
                assert!(is_bijection(&p,pat.n));assert!(score(&p)<=initial);
                if pair!=0 {assert_eq!(outputs[arm],p,"{name}: repeat gate arm");
                    assert_eq!(calls[arm],count,"{name}: repeat continuation work");}
                calls[arm]=count;lex[arm]=u64::from(do_lex);outputs[arm]=p;
            }}
        } else {minima=[0.0;2];outputs=[seed.clone(),seed];}
        differences+=usize::from(outputs[0]!=outputs[1]);
        for arm in 0..2 {totals[arm]+=minima[arm];maxima[arm]=maxima[arm].max(minima[arm]);
            recon[arm]+=calls[arm];lex_calls[arm]+=lex[arm];}
        println!("STATE_GATE_COST_ROW\t{name}\t{}\t{fill}\t{initial}\t{:.6}\t{:.6}\t{}\t{}\t{}\t{}",
            pat.n,minima[0],minima[1],calls[0],calls[1],lex[0],lex[1]);
    }
    post_core::set_priority_post_win_only(None);assert!(cache.is_empty());
    println!("STATE_GATE_COST_TOTAL eligible={eligible} old_seconds={:.6} new_seconds={:.6} old_max={:.6} new_max={:.6} old_recon={} new_recon={} old_lex={} new_lex={} different={differences}",
        totals[0],totals[1],maxima[0],maxima[1],recon[0],recon[1],lex_calls[0],lex_calls[1]);
}

#[test]
#[ignore]
fn probe_runtime_batch_campaign() {
    post_core::set_original_second_peo_rounds(std::env::var("SSI_ORIGINAL_SECOND_PEO_ROUNDS").ok()
        .map(|v|v.parse().unwrap()));
    let original=std::env::var_os("SSI_ORIGINAL_LARGE_ROOT_RESEARCH").is_some();
    set_original_large_research(if original {Some(true)} else {None});
    set_original_second_research(if std::env::var_os("SSI_ORIGINAL_SECOND_ROOT_RESEARCH").is_some() {Some(true)} else {None});
    let raw=std::env::var_os("SSI_RAW_COMPLETION_RESEARCH").is_some();
    set_raw_completion_research(raw);
    let profile=std::env::var_os("SSI_EXTENDED_TERMINAL_SCREEN").is_some();
    if profile {set_extended_terminal(Some(true));}
    let conditional=std::env::var_os("SSI_PRIORITY_POST_WIN_ONLY_SCREEN").is_some();
    if conditional {post_core::set_priority_post_win_only(Some(true));}
    let lex_conditional=std::env::var_os("SSI_LEX_AFTER_PRIORITY_WIN_SCREEN").is_some();
    if lex_conditional {set_lex_after_priority_win(Some(true));}
    let cache_path=std::env::var("SSI_BATCH_CAMPAIGN_CACHE_IN")
        .unwrap_or_else(|_|"/tmp/matrices-fast-lex-campaign-seeds.tsv".to_owned());
    let expected=std::env::var("SSI_BATCH_CAMPAIGN_EXPECTED_BASE").ok()
        .and_then(|v|v.parse::<f64>().ok()).unwrap_or(0.790519213141);
    let text=std::fs::read_to_string(cache_path).unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let reference=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/ordering/memory/evidence/0215-final-screen.tsv")).unwrap();
    let refs:std::collections::BTreeMap<String,u64>=reference.lines().skip(1).map(|line| {
        let r:Vec<_>=line.split('\t').collect();(r[0].to_owned(),r[3].parse().unwrap())
    }).collect();
    let mut sums=[0.0f64;3];let mut base=[0.0f64;3];let mut counts=[0usize;3];
    let mut wins=0;let mut losses=0;let mut seconds=0.0;let mut maximum=0.0f64;let mut output=String::new();
    for (name,pat) in crate::corpus::corpus() {
        let parent=cache.remove(&name).unwrap();let sp=scoring_pattern(&pat);let amd=refs[&name];
        take_completion_win_flags();
        let start=Instant::now();let p=order(&pat);let elapsed=start.elapsed().as_secs_f64();
        let flags=take_completion_win_flags();
        assert!(is_bijection(&p,pat.n));seconds+=elapsed;maximum=maximum.max(elapsed);
        let mut ws=scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());let old=ws.flops(&sp,&parent);let f=ws.flops(&sp,&p);
        wins+=usize::from(f<old);losses+=usize::from(f>old);let b=bucket(pat.n);counts[b]+=1;
        base[b]+=(old as f64/amd as f64).ln();sums[b]+=(f as f64/amd as f64).ln();
        output.push_str(&format!("{name}\t{}\n",p.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(",")));
        println!("BATCH_RUNTIME_ROW\t{name}\t{}\t{}\t{amd}\t{old}\t{f}\t{elapsed:.6}",pat.n,pat.nnz());
        println!("COMPLETION_STATE_ROW\t{name}\t{flags}");
    }
    assert!(cache.is_empty());assert!((aggregate(&base,&counts)-expected).abs()<1e-11);
    if let Some(path)=std::env::var_os("SSI_BATCH_CAMPAIGN_CACHE_OUT") {std::fs::write(path,output).unwrap();}
    println!("BATCH_RUNTIME_TOTAL score={:.12} wins={wins} losses={losses} seconds={seconds:.6} maximum={maximum:.6}",aggregate(&sums,&counts));
    if profile {set_extended_terminal(None);}
    if conditional {post_core::set_priority_post_win_only(None);}
    if lex_conditional {set_lex_after_priority_win(None);}
    set_raw_completion_research(false);
    set_original_large_research(None);
    set_original_second_research(None);
    post_core::set_original_second_peo_rounds(None);
}

#[test]
#[ignore]
fn probe_original_incidence_completion() {
    let cache_path=std::env::var("SSI_ORIGINAL_CAMPAIGN_CACHE_IN")
        .unwrap_or_else(|_|"/tmp/matrices-fast-lex-campaign-seeds.tsv".to_owned());
    let expected=std::env::var("SSI_ORIGINAL_CAMPAIGN_EXPECTED_BASE").ok()
        .and_then(|v|v.parse::<f64>().ok()).unwrap_or(0.790519213141);
    let text=std::fs::read_to_string(cache_path).unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let reference=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/ordering/memory/evidence/0215-final-screen.tsv")).unwrap();
    let refs:std::collections::BTreeMap<String,u64>=reference.lines().skip(1).map(|line| {
        let r:Vec<_>=line.split('\t').collect();(r[0].to_owned(),r[3].parse().unwrap())
    }).collect();
    let mut sums=[[0.0f64;3];8];let mut base=[0.0f64;3];let mut counts=[0usize;3];
    let mut wins=[0usize;8];let mut seconds=0.0;let mut maximum=0.0f64;
    for (name,pat) in crate::corpus::corpus() {
        let p=cache.remove(&name).unwrap();let sp=scoring_pattern(&pat);let amd=refs[&name];
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);let initial=score(&p);let factor=ws.borrow().nnz_l();
        let mut values=[initial;8];let start=Instant::now();
        if pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=1_300_000 &&factor<=1_000_000 &&amd<=post_core::AMD_WORK_LIMIT {
            let pp=builder.borrow_mut().permute(&p);let et=EliminationTree::from_pattern(&pp);
            let counts=column_counts_gnp(&pp,&et);
            if let Some(candidates)=peo_extract::original_candidates_with_limits(pat.n,&pat.col_ptr,&pat.row_idx,
                &pp.col_ptr,&pp.row_idx,&et.parent,&counts,&p,&[0,1,2,3],50_000,1_300_000,1_000_000) {
                for mode in 0..4 {
                    assert!(is_bijection(&candidates[mode],pat.n));let f=score(&candidates[mode]);
                    assert!(f<=initial,"{name}: original incidences {mode}");values[mode]=f;values[4+mode]=f;
                    if f<initial {if let Some(q)=post_core::refine_bounded(&pat,&candidates[mode],2,
                        50_000,1_300_000,1_000_000,&score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()) {values[4+mode]=score(&q);}}
                }
            }
        }
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        let b=if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};counts[b]+=1;base[b]+=(initial as f64/amd as f64).ln();
        for arm in 0..8 {sums[arm][b]+=(values[arm] as f64/amd as f64).ln();wins[arm]+=usize::from(values[arm]<initial);}
        println!("ORIGINAL_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{factor}\t{elapsed:.6}\t{}",
            pat.n,pat.nnz(),values.iter().map(|v|v.to_string()).collect::<Vec<_>>().join("\t"));
    }
    assert!(cache.is_empty());assert!((aggregate(&base,&counts)-expected).abs()<1e-11);
    for arm in 0..8 {println!("ORIGINAL_TOTAL arm={arm} score={:.12} wins={}",aggregate(&sums[arm],&counts),wins[arm]);}
    println!("ORIGINAL_WORK seconds={seconds:.6} maximum={maximum:.6}");
}

#[test]
#[ignore]
fn probe_selected_lex_completion() {
    let text=std::fs::read_to_string("/tmp/matrices-fast-factor1m-campaign-seeds.tsv").unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let reference=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/ordering/memory/evidence/0214-final-screen.tsv")).unwrap();
    let references:std::collections::BTreeMap<String,u64>=reference.lines().skip(1).map(|line| {
        let r:Vec<_>=line.split('\t').collect();(r[0].to_owned(),r[3].parse().unwrap())
    }).collect();
    let mut sums=[0.0f64;3];let mut counts=[0usize;3];let mut wins=0;let mut seconds=0.0;let mut maximum=0.0f64;
    let mut output=String::new();
    for (name,pat) in crate::corpus::corpus() {
        let mut p=cache.remove(&name).unwrap();let parent=p.clone();let sp=scoring_pattern(&pat);let amd=references[&name];
        let mut ws=scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());let mut builder=sorted_permutation::SortedPermutation::new(&sp);
        let initial=ws.flops(&sp,&p);let factor=ws.nnz_l();let mut f=initial;let start=Instant::now();
        let allowed=pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=1_300_000 &&factor<=1_000_000 &&amd<=post_core::AMD_WORK_LIMIT;
        if allowed {
            apply_step(&pat,&mut p,&mut f,&mut builder,&mut ws,&sp,Step::Lex(5),(50_000,1_300_000,1_000_000));
            if f<initial {apply_step(&pat,&mut p,&mut f,&mut builder,&mut ws,&sp,Step::Peo(2),(50_000,1_300_000,1_000_000));}
        }
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        let full_ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let full_builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let candidate=if amd<=post_core::AMD_WORK_LIMIT {post_core::refine_lex(&pat,&parent,
            |p|full_ws.borrow_mut().flops(&sp,p),|p|full_builder.borrow_mut().permute(p),||full_ws.borrow().nnz_l())} else {None};
        assert_eq!(p,candidate.unwrap_or(parent),"{name}: independent LexBFS/PEO chain");
        assert!(f<=initial);wins+=usize::from(f<initial);
        let bucket=if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};counts[bucket]+=1;sums[bucket]+=(f as f64/amd as f64).ln();
        output.push_str(&format!("{name}\t{}\n",p.iter().map(|v|v.to_string()).collect::<Vec<_>>().join(",")));
        println!("SELECTED_LEX_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{f}\t{factor}\t{elapsed:.6}",pat.n,pat.nnz());
    }
    assert!(cache.is_empty());let result=aggregate(&sums,&counts);assert!((result-0.790519213141).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-lex-campaign-seeds.tsv",output).unwrap();
    println!("SELECTED_LEX_TOTAL score={result:.12} wins={wins} seconds={seconds:.6} maximum={maximum:.6}");
}

#[test]
#[ignore]
fn probe_lex_completion() {
    let text=std::fs::read_to_string("/tmp/matrices-fast-factor1m-campaign-seeds.tsv").unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let reference=std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../src/ordering/memory/evidence/0214-final-screen.tsv")).unwrap();
    let references:std::collections::BTreeMap<String,u64>=reference.lines().skip(1).map(|line| {
        let r:Vec<_>=line.split('\t').collect();(r[0].to_owned(),r[3].parse().unwrap())
    }).collect();
    let mut sums=vec![[0.0f64;3];18];let mut base=[0.0f64;3];let mut counts=[0usize;3];
    let mut wins=[0usize;18];let mut seconds=0.0;let mut maximum=0.0f64;
    for (name,pat) in crate::corpus::corpus() {
        let p=cache.remove(&name).unwrap();let sp=scoring_pattern(&pat);let amd=references[&name];
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let initial=score(&p);let factor=ws.borrow().nnz_l();let mut values=[initial;18];
        let start=Instant::now();
        if pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=1_300_000 &&factor<=1_000_000 &&amd<=post_core::AMD_WORK_LIMIT {
            let pp=builder.borrow_mut().permute(&p);let et=EliminationTree::from_pattern(&pp);
            let counts=column_counts_gnp(&pp,&et);
            let degrees:Vec<usize>=pat.col_ptr.windows(2).map(|a|a[1]-a[0]).collect();
            if let Some(candidates)=peo_extract::lex_candidates_with_limits(pat.n,&pp.col_ptr,&pp.row_idx,
                &et.parent,&counts,&p,&degrees,&[0,1,2,3,4,5],50_000,1_300_000,1_000_000) {
                for mode in 0..6 {
                    assert!(is_bijection(&candidates[mode],pat.n));
                    let f=score(&candidates[mode]);assert!(f<=initial,"{name}: LexBFS {mode}");
                    values[mode]=f;values[6+mode]=f;
                    if f<initial {
                        if let Some(q)=post_core::refine_bounded(&pat,&candidates[mode],2,50_000,1_300_000,1_000_000,
                            &score,|p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()) {values[6+mode]=score(&q);}
                    }
                }
                for pair in 0..3 {
                    let mode=if values[2*pair]<=values[2*pair+1] {2*pair} else {2*pair+1};
                    values[12+pair]=values[mode];values[15+pair]=values[6+mode];
                }
            }
        }
        let elapsed=start.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        let bucket=if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};counts[bucket]+=1;
        base[bucket]+=(initial as f64/amd as f64).ln();
        for arm in 0..18 {sums[arm][bucket]+=(values[arm] as f64/amd as f64).ln();wins[arm]+=usize::from(values[arm]<initial);}
        println!("LEX_ROW\t{name}\t{}\t{}\t{amd}\t{initial}\t{factor}\t{elapsed:.6}\t{}",
            pat.n,pat.nnz(),values.iter().map(|v|v.to_string()).collect::<Vec<_>>().join("\t"));
    }
    assert!(cache.is_empty());let baseline=aggregate(&base,&counts);assert!((baseline-0.790704642081).abs()<1e-11);
    for arm in 0..18 {println!("LEX_TOTAL arm={arm} score={:.12} wins={}",aggregate(&sums[arm],&counts),wins[arm]);}
    println!("LEX_WORK seconds={seconds:.6} maximum={maximum:.6}");
}

#[test]
#[ignore]
fn probe_selected_completion_priority() {
    let text=std::fs::read_to_string("/tmp/matrices-fast-wide-priority-campaign-seeds.tsv").unwrap();
    let mut control:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
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
        if pat.n>=6 &&pat.n<=50_000 &&pat.nnz()<=1_300_000 &&factor<=1_000_000 &&initial<=20_000_000_000 {
            for step in [Step::Priority(1,2),Step::Peo(2)] {
                apply_step(&pat,&mut p,&mut f,&mut builder,&mut ws,&sp,step,(50_000,1_300_000,1_000_000));
            }
        }
        let elapsed=t.elapsed().as_secs_f64();seconds+=elapsed;maximum=maximum.max(elapsed);
        let full_ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let full_builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let candidate=post_core::refine_priority(&pat,&parent,|p|full_ws.borrow_mut().flops(&sp,p),
            |p|full_builder.borrow_mut().permute(p),||full_ws.borrow().nnz_l()).unwrap_or(parent);
        assert_eq!(p,candidate,"{name}: independent chain vs selected helper");
        let control=control.remove(&name).unwrap();let control_f=ws.flops(&sp,&control);
        assert!(f<=control_f,"{name}: preserve submitted 300k score");
        if pat.n<=30_000 &&pat.nnz()<=180_000 &&factor<=300_000 {
            assert_eq!(p,control,"{name}: preserve common 300k permutation");
        }
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
    assert!(control.is_empty());
    assert!((result-0.790704642081).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-factor1m-campaign-seeds.tsv",output).unwrap();
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
        Step::Lex(mode) => {
            let pp=builder.permute(p);let et=EliminationTree::from_pattern(&pp);
            let counts=column_counts_gnp(&pp,&et);
            let degrees:Vec<_>=pat.col_ptr.windows(2).map(|a|a[1]-a[0]).collect();
            if let Some(candidates)=peo_extract::lex_candidates_with_limits(pat.n,&pp.col_ptr,&pp.row_idx,
                &et.parent,&counts,p,&degrees,&[mode],limits.0,limits.1,limits.2) {
                for candidate in candidates {
                    assert!(is_bijection(&candidate,pat.n));let nf=ws.flops(sp,&candidate);
                    assert!(nf<=*f);if nf<*f {*p=candidate;*f=nf;}
                }
            }
        },
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
    let state_mode=std::env::var_os("SSI_COMPARE_STATE_GATED_TERMINAL").is_some();
    let frontier_mode=std::env::var_os("SSI_COMPARE_VALIDATED_FRONTIER").is_some();
    let budget_mode=std::env::var_os("SSI_COMPARE_CORE_WINDOW_BUDGET").is_some();
    let original_mode=std::env::var_os("SSI_COMPARE_ORIGINAL_LARGE").is_some();
    let second_mode=std::env::var_os("SSI_COMPARE_ORIGINAL_SECOND").is_some();
    let combined_mode=std::env::var_os("SSI_COMPARE_ORIGINAL_COMBINED").is_some();
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
    for grid in [32usize,64,96,128,192] {
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
        let mut medium_drops=[0u64;2];
        let (cp,ri)=core_of(&pat);
        let core=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize>=feral_amd::amd_order(&core).unwrap().into_iter().map(|v|v as usize).collect();
        let amd_flops=ws.flops(&sp,&amd);
        for pair in 0..2 { for offset in 0..2 {
            let arm = (pair+offset)%2;
            indep_first::set_extra_enabled(true);
            indep_first::set_search_enabled(true);
            post_core::set_priority_enabled(true);
            post_core::set_priority_wide(true);
            peo_extract::set_indexed_priority(true);
            post_core::set_lex_enabled(true);
            set_core_window_exclusion(if budget_mode {Some(arm==1)} else {None});
            set_original_large_research(Some(second_mode ||(combined_mode ||original_mode) &&arm==1));
            set_original_second_research(Some((second_mode ||combined_mode) &&arm==1));
            set_medium_batch_enabled(if budget_mode ||original_mode ||second_mode ||combined_mode {false} else if frontier_mode {arm==0} else {cap_mode ||state_mode ||arm==1});
            take_medium_batch_drops();set_raw_completion_research(frontier_mode &&arm==0);
            set_extended_terminal(if state_mode {Some(arm==1)} else {None});
            post_core::set_priority_post_win_only(if state_mode {Some(arm==1)} else {None});
            set_lex_after_priority_win(if state_mode {Some(arm==1)} else {None});
            if cap_mode {std::env::set_var("SSI_LADDER_CAP",if arm==0 {"64"} else {"32"});}
            let t = Instant::now(); let p = order(&pat);
            minima[arm] = minima[arm].min(t.elapsed().as_secs_f64());
            let drops=take_medium_batch_drops();
            if pair!=0 {assert_eq!(drops,medium_drops[arm],"{name}: deterministic batch drops");}
            medium_drops[arm]=drops;
            assert!(is_bijection(&p,pat.n));
            if pair!=0 { assert_eq!(p,outputs[arm],"{name} arm={arm}"); }
            outputs[arm] = p; flops[arm] = ws.flops(&sp,&outputs[arm]);
            if arm==0 { old_factor = ws.nnz_l(); }
        } }
        assert!(flops[0]<=amd_flops &&flops[1]<=amd_flops,"{name}: AMD fallback floor");
        if !cap_mode &&!state_mode &&!frontier_mode &&!budget_mode &&!original_mode &&!second_mode &&!combined_mode &&medium_drops[1]==0 {
            assert_eq!(outputs[0],outputs[1],"{name}: no medium batch truncated");
        }
        println!("CAMPAIGN_STRESS\t{name}\t{}\t{}\t{:.6}\t{:.6}\t{}\t{}\t{old_factor}",
            pat.n,pat.nnz(),minima[0],minima[1],flops[0],flops[1]);
        println!("MEDIUM_BATCH_STRESS\t{name}\t{amd_flops}\t{}\t{}",medium_drops[0],medium_drops[1]);
    }
    indep_first::set_extra_enabled(true);
    indep_first::set_search_enabled(true);
    post_core::set_priority_enabled(true);
    post_core::set_priority_wide(true);
    peo_extract::set_indexed_priority(true);
    post_core::set_lex_enabled(true);
    set_medium_batch_enabled(false);take_medium_batch_drops();set_raw_completion_research(false);
    set_extended_terminal(None);post_core::set_priority_post_win_only(None);
    set_lex_after_priority_win(None);
    set_core_window_exclusion(None);
    set_original_large_research(None);
    set_original_second_research(None);
    if cap_mode {
        if let Some(value)=prior_cap {std::env::set_var("SSI_LADDER_CAP",value);}
        else {std::env::remove_var("SSI_LADDER_CAP");}
    }
}
