//! Test-only marginal screen against the completed compact-core candidate.
use super::*;
use crate::ordering::{metric_sweep,post_core,scoring_ws,sorted_permutation};
use std::time::Instant;

#[test]
#[ignore]
fn probe_bounded_generic_cores() {
    const ARMS:usize=47;
    let text=std::fs::read_to_string(std::env::var("SSI_CAMPAIGN_SEED_CACHE").unwrap()).unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let mut sums=[[0.0;3];ARMS+1];let mut counts=[0;3];let mut wins=[0;ARMS];
    let mut seconds=[0.0;ARMS];let mut maximum=[0.0f64;ARMS];let mut output=String::new();
    for (name,pat) in crate::corpus::corpus() {
        let p=cache.remove(&name).unwrap();
        let sp=ScoringPattern{n:pat.n,col_ptr:pat.col_ptr.clone(),row_idx:pat.row_idx.clone()};
        let ws=std::cell::RefCell::new(scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz()));
        let builder=std::cell::RefCell::new(sorted_permutation::SortedPermutation::new(&sp));
        let score=|p:&[usize]|ws.borrow_mut().flops(&sp,p);
        let base=score(&p);let factor=ws.borrow().nnz_l();
        let cp:Vec<i32>=pat.col_ptr.iter().map(|&v|v as i32).collect();
        let ri:Vec<i32>=pat.row_idx.iter().map(|&v|v as i32).collect();
        let raw=feral_ordering_core::CscPattern::new(pat.n,&cp,&ri).unwrap();
        let amd:Vec<usize>=run_pass(&raw,Pass::Amd).unwrap().into_iter().map(|v|v as usize).collect();
        let reference=score(&amd);let mut row=[base;ARMS];let mut costs=[0.0;ARMS];
        let mut union=p;let mut union_f=base;
        if pat.n>=32 &&pat.n<=18_000 &&pat.nnz()<=80_000 &&factor<=200_000 &&base<=20_000_000_000 {
            if let Some((_,_,cores))=run_with_cores(&sp,8_000_000) {
                for (rank,(amd,lc)) in cores.cores.iter().enumerate() {
                    if amd.saturating_mul(2)>base.saturating_mul(3) {continue;}
                    let core=feral_ordering_core::CscPattern::new(lc.core_pat.n,&lc.ccp,&lc.cri).unwrap();
                    let allowance=base.saturating_mul(4).clamp(1_000_000,80_000_000);
                    for (m,spec) in metric_sweep::EXTRA_METRICS.iter().enumerate() {
                        for (a,alpha) in [10.0,2.5,1.0].into_iter().enumerate() {
                            let arm=3*m+a;let t=Instant::now();let mut f=u64::MAX;let mut completed=false;
                            if let Some(p)=metric_sweep::order_generic_limited(&core,alpha,true,spec,allowance) {
                                completed=true;let p:Vec<usize>=p.into_iter().map(|v|v as usize).collect();
                                let p=splice(&lc.il,&p);assert!(crate::ordering::is_bijection(&p,pat.n));f=score(&p);
                                if f<union_f {union_f=f;union=p;}
                            }
                            let elapsed=t.elapsed().as_secs_f64();row[arm]=row[arm].min(f);costs[arm]+=elapsed;
                            println!("GENERIC_CANDIDATE\t{name}\t{rank}\t{arm}\t{}\t{alpha}\t{allowance}\t{}\t{f}\t{elapsed:.6}",spec.name,usize::from(completed));
                        }
                    }
                }
            }
            row[45]=union_f;costs[45]=costs[..45].iter().sum();row[46]=union_f;costs[46]=costs[45];
            if union_f<base {
                let t=Instant::now();
                if let Some(p)=post_core::refine(&pat,&union,1,200_000,&score,
                    |p|builder.borrow_mut().permute(p),||ws.borrow().nnz_l()) {row[46]=score(&p);union=p;}
                costs[46]+=t.elapsed().as_secs_f64();
            }
        }
        output.push_str(&name);output.push('\t');
        for (i,v) in union.iter().enumerate() {if i!=0 {output.push(',');}output.push_str(&v.to_string());}
        output.push('\n');
        let b=if pat.n<1_000 {0} else if pat.n<10_000 {1} else {2};counts[b]+=1;
        sums[0][b]+=(base as f64/reference as f64).ln();
        print!("GENERIC_ROW\t{name}\t{}\t{}\t{reference}\t{base}\t{factor}",pat.n,pat.nnz());
        for arm in 0..ARMS {
            assert!(row[arm]<=base);sums[arm+1][b]+=(row[arm] as f64/reference as f64).ln();
            wins[arm]+=usize::from(row[arm]<base);seconds[arm]+=costs[arm];maximum[arm]=maximum[arm].max(costs[arm]);
            print!("\t{}",row[arm]);
        }println!();
    }
    assert!(cache.is_empty());
    let aggregate=|s:&[f64;3]|[0.30,0.30,0.40].iter().enumerate()
        .map(|(i,w)|w*(s[i]/counts[i] as f64).exp()).sum::<f64>();
    assert!((aggregate(&sums[0])-0.791316868681).abs()<1e-11);
    std::fs::write("/tmp/matrices-fast-generic-union-campaign-seeds.tsv",output).unwrap();
    for arm in 0..ARMS {println!("GENERIC_TOTAL arm={arm} score={:.12} wins={} seconds={:.6} max={:.6}",
        aggregate(&sums[arm+1]),wins[arm],seconds[arm],maximum[arm]);}
}
