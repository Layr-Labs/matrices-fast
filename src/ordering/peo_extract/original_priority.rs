//! Maximum-cardinality ties using visited or remaining original incidences.
use super::Completion;
const NONE:usize=usize::MAX;

struct Queue {heap:Vec<usize>,position:Vec<usize>,key:Vec<u128>}
impl Queue {
    fn new(priority:&[u64],leading:&[usize])->Self {
        let n=priority.len();let mut heap:Vec<_>=(0..n).collect();
        heap.sort_unstable_by_key(|&v|(priority[v],v));
        let mut key=vec![0u128;n];
        for (rank,&v) in heap.iter().enumerate() {key[v]=((leading[v] as u128)<<32)|rank as u128;}
        heap.reverse();let mut position=vec![0usize;n];
        for (i,&v) in heap.iter().enumerate() {position[v]=i;}
        let mut q=Self{heap,position,key};
        for hole in (0..n/2).rev() {let v=q.heap[hole];q.down(hole,v);}
        q
    }
    #[inline]
    fn down(&mut self,mut hole:usize,v:usize) {
        loop {
            let left=hole*2+1;if left>=self.heap.len() {break;}
            let right=left+1;let child=if right<self.heap.len() &&self.key[self.heap[right]]>self.key[self.heap[left]] {right} else {left};
            let u=self.heap[child];if self.key[u]<=self.key[v] {break;}
            self.heap[hole]=u;self.position[u]=hole;hole=child;
        }
        self.heap[hole]=v;self.position[v]=hole;
    }
    #[inline]
    fn up(&mut self,mut hole:usize,v:usize) {
        while hole!=0 {
            let parent=(hole-1)/2;let u=self.heap[parent];if self.key[u]>=self.key[v] {break;}
            self.heap[hole]=u;self.position[u]=hole;hole=parent;
        }
        self.heap[hole]=v;self.position[v]=hole;
    }
    #[inline]
    fn update(&mut self,v:usize,bit:u32,decrease:bool) {
        let hole=self.position[v];if hole==NONE {return;}
        if decrease {self.key[v]-=1u128<<bit;self.down(hole,v);}
        else {self.key[v]+=1u128<<bit;self.up(hole,v);}
    }
    #[inline]
    fn pop(&mut self)->Option<usize> {
        let &v=self.heap.first()?;let last=self.heap.pop().unwrap();self.position[v]=NONE;
        if !self.heap.is_empty() {self.down(0,last);}Some(v)
    }
}

fn priorities(adj:&Completion,cp:&[usize],incumbent:&[usize],mode:usize)->Vec<u64> {
    let n=adj.len();let mut priority=vec![0u64;n];
    for (pos,&v) in incumbent.iter().enumerate() {
        let value=if mode<2 {cp[v+1]-cp[v]} else {n.saturating_sub(adj[v].len())};
        priority[v]=((value as u64)<<32)|(n-pos) as u64;
    }
    priority
}

pub(super) fn order(adj:&Completion,cp:&[usize],ri:&[usize],incumbent:&[usize],mode:usize)->Vec<usize> {
    let n=adj.len();let decrease=mode%2!=0;
    let leading:Vec<_>=(0..n).map(|v|if decrease {ri.len()+cp[v+1]-cp[v]} else {0}).collect();
    let mut queue=Queue::new(&priorities(adj,cp,incumbent,mode),&leading);
    let mut out=Vec::with_capacity(n);
    while let Some(v)=queue.pop() {
        out.push(v);if out.len()==n {break;}
        for &u in &ri[cp[v]..cp[v+1]] {queue.update(u,32,decrease);}
        for &u in &adj[v] {queue.update(u as usize,64,false);}
    }
    out.reverse();out
}

#[cfg(test)]
pub(super) fn reference(adj:&Completion,cp:&[usize],ri:&[usize],incumbent:&[usize],mode:usize)->Vec<usize> {
    let n=adj.len();let priority=priorities(adj,cp,incumbent,mode);let decrease=mode%2!=0;
    let mut primary=vec![0usize;n];let mut visited=vec![false;n];
    let mut secondary:Vec<_>=(0..n).map(|v|if decrease {ri.len()+cp[v+1]-cp[v]} else {0}).collect();
    let mut out=Vec::with_capacity(n);
    while out.len()<n {
        let v=(0..n).filter(|&v|!visited[v]).max_by_key(|&v|(primary[v],secondary[v],priority[v],v)).unwrap();
        visited[v]=true;out.push(v);
        for &u in &ri[cp[v]..cp[v+1]] {if !visited[u] {if decrease {secondary[u]-=1;} else {secondary[u]+=1;}}}
        for &u in &adj[v] {let u=u as usize;if !visited[u] {primary[u]+=1;}}
    }
    out.reverse();out
}
