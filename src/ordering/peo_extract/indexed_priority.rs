//! Eager maximum-cardinality queue: one live item per vertex.
use super::Completion;

const ABSENT:usize=usize::MAX;

struct Queue {
    heap:Vec<usize>,
    position:Vec<usize>,
    key:Vec<u64>,
}

impl Queue {
    fn new(priority:&[u64])->Self {
        let n=priority.len();assert!(n<=u32::MAX as usize);
        let mut heap:Vec<usize>=(0..n).collect();
        heap.sort_unstable_by_key(|&v|(priority[v],v));
        let mut key=vec![0;n];
        for (rank,&v) in heap.iter().enumerate() {key[v]=rank as u64;}
        heap.reverse();
        let mut position=vec![0;n];
        for (i,&v) in heap.iter().enumerate() {position[v]=i;}
        Self {heap,position,key}
    }

    #[inline]
    fn pop(&mut self)->Option<usize> {
        let &v=self.heap.first()?;
        let last=self.heap.pop().unwrap();self.position[v]=ABSENT;
        if !self.heap.is_empty() {
            let mut hole=0;
            loop {
                let left=2*hole+1;if left>=self.heap.len() {break;}
                let right=left+1;
                let child=if right<self.heap.len() &&self.key[self.heap[right]]>self.key[self.heap[left]] {right} else {left};
                let u=self.heap[child];if self.key[u]<=self.key[last] {break;}
                self.heap[hole]=u;self.position[u]=hole;hole=child;
            }
            self.heap[hole]=last;self.position[last]=hole;
        }
        Some(v)
    }

    #[inline]
    fn increase(&mut self,v:usize) {
        let mut hole=self.position[v];if hole==ABSENT {return;}
        self.key[v]+=1u64<<32;
        while hole!=0 {
            let parent=(hole-1)/2;let u=self.heap[parent];
            if self.key[u]>=self.key[v] {break;}
            self.heap[hole]=u;self.position[u]=hole;hole=parent;
        }
        self.heap[hole]=v;self.position[v]=hole;
    }
}

pub(super) fn order(adj:&Completion,priority:&[u64])->Vec<usize> {
    let mut queue=Queue::new(priority);
    let mut out=Vec::with_capacity(adj.len());
    while let Some(v)=queue.pop() {
        out.push(v);if out.len()==adj.len() {break;}
        for &u in &adj[v] {queue.increase(u as usize);}
    }
    out.reverse();out
}

#[cfg(test)]
#[test]
#[ignore]
fn public_kernel_comparison() {
    use crate::ordering::*;
    let text=std::fs::read_to_string("/tmp/matrices-fast-compact-core-campaign-seeds.tsv").unwrap();
    let mut cache:std::collections::BTreeMap<String,Vec<usize>>=text.lines().map(|line| {
        let (name,p)=line.split_once('\t').unwrap();
        (name.to_owned(),p.split(',').map(|v|v.parse().unwrap()).collect())
    }).collect();
    let mut totals=[0.0;2];let mut maxima=[0.0f64;2];let mut cases=0;
    for (name,pat) in crate::corpus::corpus() {
        let p=cache.remove(&name).unwrap();
        if pat.n<6 ||pat.n>50_000 ||pat.nnz()>1_300_000 {continue;}
        let sp=ScoringPattern{n:pat.n,col_ptr:pat.col_ptr.clone(),row_idx:pat.row_idx.clone()};
        let mut ws=scoring_ws::ScoreWorkspace::new(pat.n,pat.nnz());
        if ws.flops(&sp,&p)>20_000_000_000 {continue;}
        let factor=ws.nnz_l();let pp=permute_pattern(&sp,&p);
        let et=EliminationTree::from_pattern(&pp);let counts=column_counts_gnp(&pp,&et);
        let Some(adj)=super::reconstruct(pat.n,&pp.col_ptr,&pp.row_idx,&et.parent,
            &counts,&p,50_000,1_300_000,2_000_000) else {continue;};
        let degrees:Vec<usize>=pat.col_ptr.windows(2).map(|a|a[1]-a[0]).collect();
        let mut outputs=[Vec::new(),Vec::new()];let mut minima=[f64::MAX;2];
        for pair in 0..2 {for offset in 0..2 {
            let arm=(pair+offset)%2;super::set_indexed_priority(arm==1);
            let start=std::time::Instant::now();
            let candidate=super::priority_mcs(&adj,&p,&counts,&degrees,1);
            minima[arm]=minima[arm].min(start.elapsed().as_secs_f64());
            if pair!=0 {assert_eq!(candidate,outputs[arm],"{name}: repeated kernel");}
            outputs[arm]=candidate;
        }}
        assert_eq!(outputs[0],outputs[1],"{name}: indexed kernel");
        for arm in 0..2 {totals[arm]+=minima[arm];maxima[arm]=maxima[arm].max(minima[arm]);}
        cases+=1;
        println!("INDEXED_PRIORITY_ROW\t{name}\t{}\t{}\t{factor}\t{:.6}\t{:.6}",pat.n,pat.nnz(),minima[0],minima[1]);
    }
    super::set_indexed_priority(true);assert!(cache.is_empty());assert!(cases>0);
    println!("INDEXED_PRIORITY_TOTAL cases={cases} old={:.6} new={:.6} oldmax={:.6} newmax={:.6}",totals[0],totals[1],maxima[0],maxima[1]);
}
