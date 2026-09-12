//! Ordered partition refinement for lexicographic breadth-first search.
use super::Completion;
const NONE:usize=usize::MAX;

struct Part {head:usize,prev:usize,next:usize,stamp:usize,split:usize}

pub(super) fn order(adj:&Completion,initial:&[usize],reverse_neighbors:bool)->Vec<usize> {
    let n=adj.len();if n==0 {return Vec::new();}
    let mut parts:Vec<Part>=(0..=n).map(|_|Part{head:NONE,prev:NONE,next:NONE,stamp:0,split:NONE}).collect();
    let mut free:Vec<usize>=(1..=n).collect();
    let mut group=vec![0usize;n];let mut prev=vec![NONE;n];let mut next=vec![NONE;n];
    for (i,&v) in initial.iter().enumerate() {
        if i!=0 {prev[v]=initial[i-1];}
        if i+1<n {next[v]=initial[i+1];}
    }
    parts[0].head=initial[0];let mut first=0;
    let mut out=Vec::with_capacity(n);
    while out.len()<n {
        let b=first;let v=parts[b].head;let following=next[v];parts[b].head=following;
        if following!=NONE {prev[following]=NONE;}
        group[v]=NONE;out.push(v);
        if following==NONE {remove(&mut parts,b,&mut first);free.push(b);}
        if out.len()==n {break;}
        let stamp=out.len();
        let neighbors=&adj[v];
        for offset in 0..neighbors.len() {
            let i=if reverse_neighbors {neighbors.len()-1-offset} else {offset};
            let u=neighbors[i] as usize;let b=group[u];if b==NONE {continue;}
            let split=if parts[b].stamp==stamp {parts[b].split} else {
                let s=free.pop().expect("a new part has an available vertex slot");
                let before=parts[b].prev;
                parts[s]=Part{head:NONE,prev:before,next:b,stamp,split:s};
                if before==NONE {first=s;} else {parts[before].next=s;}
                parts[b].prev=s;parts[b].stamp=stamp;parts[b].split=s;s
            };
            if split==b {continue;}
            let before=prev[u];let after=next[u];
            if before==NONE {parts[b].head=after;} else {next[before]=after;}
            if after!=NONE {prev[after]=before;}
            let head=parts[split].head;prev[u]=NONE;next[u]=head;
            if head!=NONE {prev[head]=u;}parts[split].head=u;group[u]=split;
            if parts[b].head==NONE {remove(&mut parts,b,&mut first);free.push(b);}
        }
    }
    out.reverse();out
}

fn remove(parts:&mut [Part],b:usize,first:&mut usize) {
    let before=parts[b].prev;let after=parts[b].next;
    if before==NONE {*first=after;} else {parts[before].next=after;}
    if after!=NONE {parts[after].prev=before;}
}

#[cfg(test)]
pub(super) fn reference(adj:&Completion,initial:&[usize],reverse_neighbors:bool)->Vec<usize> {
    let n=adj.len();let mut labels=vec![Vec::<usize>::new();n];
    let mut tie=vec![0usize;n];let mut visited=vec![false;n];
    for (pos,&v) in initial.iter().enumerate() {tie[v]=n-pos;}
    let mut serial=n;let mut out=Vec::with_capacity(n);
    while out.len()<n {
        let v=(0..n).filter(|&v|!visited[v]).max_by(|&a,&b|
            labels[a].cmp(&labels[b]).then(tie[a].cmp(&tie[b])).then(a.cmp(&b))).unwrap();
        visited[v]=true;out.push(v);let neighbors=&adj[v];
        for offset in 0..neighbors.len() {
            let i=if reverse_neighbors {neighbors.len()-1-offset} else {offset};
            let u=neighbors[i] as usize;if visited[u] {continue;}
            labels[u].push(n-out.len());serial+=1;tie[u]=serial;
        }
    }
    out.reverse();out
}
