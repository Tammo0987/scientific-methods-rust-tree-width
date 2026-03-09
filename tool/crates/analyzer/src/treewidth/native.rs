use crate::graph::Graph;

pub fn compute(g: &Graph) -> u32 {
    if g.n <= 1 || g.edge_count() == 0 {
        return 0;
    }
    min_degree(g).min(min_fill(g))
}

fn min_degree(g: &Graph) -> u32 {
    let mut adj = sorted_adj(g);
    let mut active = vec![true; g.n];
    let mut tw = 0u32;

    for _ in 0..g.n {
        let v = (0..g.n)
            .filter(|&v| active[v])
            .min_by_key(|&v| adj[v].len())
            .unwrap();

        tw = tw.max(adj[v].len() as u32);
        contract(&mut adj, v);
        active[v] = false;
    }
    tw
}

fn min_fill(g: &Graph) -> u32 {
    let mut adj = sorted_adj(g);
    let mut active = vec![true; g.n];
    let mut tw = 0u32;

    for _ in 0..g.n {
        let v = (0..g.n)
            .filter(|&v| active[v])
            // FlowCutter's exact criterion: fill-in count weighted heavily,
            // degree as tiebreaker (greedy_order.cpp:167).
            .min_by_key(|&v| fill_count(v, &adj) * 100 + adj[v].len())
            .unwrap();

        tw = tw.max(adj[v].len() as u32);
        contract(&mut adj, v);
        active[v] = false;
    }
    tw
}

fn contract(adj: &mut Vec<Vec<usize>>, v: usize) {
    // mem::take empties adj[v] and returns its contents without cloning,
    // which lets us borrow adj mutably for the neighbour updates below.
    let v_nb = std::mem::take(&mut adj[v]);

    for &x in &v_nb {
        let x_nb = std::mem::take(&mut adj[x]);
        adj[x] = sorted_union(&v_nb, &x_nb, v, x);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
