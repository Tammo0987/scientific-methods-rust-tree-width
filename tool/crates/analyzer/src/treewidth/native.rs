use crate::graph::Graph;

pub fn compute(g: &Graph) -> u32 {
    if g.n <= 1 || g.edge_count() == 0 {
        return 0;
    }
    min_degree(g).min(min_fill(g))
}

fn min_degree(g: &Graph) -> u32 {
    let mut adj = sorted_adj(g);
    let mut heap = MinIdHeap::with_capacity(g.n);
    let mut tw = 0u32;

    for v in 0..g.n {
        heap.push(v, adj[v].len());
    }

    while let Some(v) = heap.pop() {
        tw = tw.max(adj[v].len() as u32);
        for y in contract(&mut adj, v) {
            heap.push_or_set_key(y, adj[y].len());
        }
    }
    tw
}

fn min_fill(g: &Graph) -> u32 {
    let mut adj = sorted_adj(g);
    let mut heap = MinIdHeap::with_capacity(g.n);
    let mut tw = 0u32;

    for v in 0..g.n {
        heap.push(v, shortcut_key(v, &adj));
    }

    while let Some(v) = heap.pop() {
        tw = tw.max(adj[v].len() as u32);
        for y in contract(&mut adj, v) {
            heap.push_or_set_key(y, shortcut_key(y, &adj));
        }
    }
    tw
}

fn shortcut_key(v: usize, adj: &[Vec<usize>]) -> usize {
    fill_count(v, adj) * 100 + adj[v].len()
}

fn contract(adj: &mut Vec<Vec<usize>>, v: usize) -> Vec<usize> {
    // mem::take empties adj[v] and returns its contents without cloning,
    // which lets us borrow adj mutably for the neighbour updates below.
    let v_nb = std::mem::take(&mut adj[v]);

    for &x in &v_nb {
        let x_nb = std::mem::take(&mut adj[x]);
        adj[x] = sorted_union(&v_nb, &x_nb, v, x);
    }

    v_nb
}

fn fill_count(v: usize, adj: &[Vec<usize>]) -> usize {
    let nb = &adj[v];
    let mut count = 0;
    for i in 0..nb.len() {
        for j in (i + 1)..nb.len() {
            if adj[nb[i]].binary_search(&nb[j]).is_err() {
                count += 1;
            }
        }
    }
    count
}

fn sorted_union(a: &[usize], b: &[usize], exclude1: usize, exclude2: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let mut ai = 0;
    let mut bi = 0;

    while ai < a.len() || bi < b.len() {
        let pick = match (ai < a.len(), bi < b.len()) {
            (true, false) => {
                let v = a[ai];
                ai += 1;
                v
            }
            (false, true) => {
                let v = b[bi];
                bi += 1;
                v
            }
            (true, true) => match a[ai].cmp(&b[bi]) {
                std::cmp::Ordering::Less => {
                    let v = a[ai];
                    ai += 1;
                    v
                }
                std::cmp::Ordering::Greater => {
                    let v = b[bi];
                    bi += 1;
                    v
                }
                std::cmp::Ordering::Equal => {
                    let v = a[ai];
                    ai += 1;
                    bi += 1;
                    v
                }
            },
            (false, false) => unreachable!(),
        };
        if pick != exclude1 && pick != exclude2 {
            out.push(pick);
        }
    }
    out
}

fn sorted_adj(g: &Graph) -> Vec<Vec<usize>> {
    g.adj
        .iter()
        .map(|nb| {
            let mut v = nb.clone();
            v.sort_unstable();
            v
        })
        .collect()
}

struct MinIdHeap {
    heap: Vec<HeapEntry>,
    id_pos: Vec<usize>,
    contained: Vec<bool>,
}

#[derive(Clone, Copy)]
struct HeapEntry {
    id: usize,
    key: usize,
}

impl MinIdHeap {
    fn with_capacity(id_count: usize) -> Self {
        Self {
            heap: Vec::with_capacity(id_count),
            id_pos: vec![usize::MAX; id_count],
            contained: vec![false; id_count],
        }
    }

    fn push(&mut self, id: usize, key: usize) {
        debug_assert!(!self.contained[id]);
        self.contained[id] = true;
        self.heap.push(HeapEntry { id, key });
        let pos = self.heap.len() - 1;
        self.id_pos[id] = pos;
        self.move_up(pos);
    }

    fn pop(&mut self) -> Option<usize> {
        if self.heap.is_empty() {
            return None;
        }

        let ret = self.heap[0].id;
        self.contained[ret] = false;
        self.id_pos[ret] = usize::MAX;

        if self.heap.len() == 1 {
            self.heap.pop();
            return Some(ret);
        }

        let last = self.heap.pop().unwrap();
        self.heap[0] = last;
        self.id_pos[last.id] = 0;
        self.move_down(0);
        Some(ret)
    }

    fn push_or_set_key(&mut self, id: usize, key: usize) {
        if !self.contained[id] {
            self.push(id, key);
            return;
        }

        let pos = self.id_pos[id];
        let old = self.heap[pos].key;
        if old < key {
            self.heap[pos].key = key;
            self.move_down(pos);
        } else if key < old {
            self.heap[pos].key = key;
            self.move_up(pos);
        }
    }

    fn move_up(&mut self, mut pos: usize) {
        if pos == 0 {
            return;
        }

        let entry = self.heap[pos];
        while pos != 0 {
            let parent = (pos - 1) / 4;
            if !(entry.key < self.heap[parent].key) {
                break;
            }
            self.heap[pos] = self.heap[parent];
            self.id_pos[self.heap[pos].id] = pos;
            pos = parent;
        }
        self.heap[pos] = entry;
        self.id_pos[entry.id] = pos;
    }

    fn move_down(&mut self, mut pos: usize) {
        let entry = self.heap[pos];
        loop {
            let begin = 4 * pos + 1;
            if begin >= self.heap.len() {
                break;
            }

            let end = (begin + 4).min(self.heap.len());
            let mut min_child = begin;
            for i in (begin + 1)..end {
                if self.heap[i].key < self.heap[min_child].key {
                    min_child = i;
                }
            }

            if !(self.heap[min_child].key < entry.key) {
                break;
            }

            self.heap[pos] = self.heap[min_child];
            self.id_pos[self.heap[pos].id] = pos;
            pos = min_child;
        }

        self.heap[pos] = entry;
        self.id_pos[entry.id] = pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(has_flowcutter_ffi)]
    use crate::treewidth::compute_oracle;

    fn g(n: usize, edges: &[(usize, usize)]) -> Graph {
        Graph::from_edges(n, edges)
    }

    #[test]
    fn empty_and_trivial() {
        assert_eq!(compute(&g(0, &[])), 0);
        assert_eq!(compute(&g(1, &[])), 0);
        assert_eq!(compute(&g(5, &[])), 0);
    }

    #[test]
    fn path_tw1() {
        assert_eq!(compute(&g(4, &[(0, 1), (1, 2), (2, 3)])), 1);
    }

    #[test]
    fn tree_tw1() {
        assert_eq!(compute(&g(5, &[(0, 1), (0, 2), (0, 3), (3, 4)])), 1);
    }

    #[test]
    fn cycle_c4_tw2() {
        assert_eq!(compute(&g(4, &[(0, 1), (1, 2), (2, 3), (3, 0)])), 2);
    }

    #[test]
    fn complete_k4_tw3() {
        assert_eq!(
            compute(&g(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)])),
            3
        );
    }

    #[cfg(has_flowcutter_ffi)]
    #[test]
    fn matches_flowcutter_oracle_on_reference_graphs() {
        let graphs = [
            g(4, &[(0, 1), (1, 2), (2, 3)]),
            g(4, &[(0, 1), (1, 2), (2, 3), (3, 0)]),
            g(5, &[(0, 1), (0, 2), (0, 3), (3, 4)]),
            g(6, &[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5)]),
            g(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)]),
            g(5, &[(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (2, 3)]),
        ];

        for graph in graphs {
            let native = compute(&graph);
            let oracle = compute_oracle(&graph).expect("oracle should succeed");
            assert_eq!(native, oracle);
        }
    }
}
