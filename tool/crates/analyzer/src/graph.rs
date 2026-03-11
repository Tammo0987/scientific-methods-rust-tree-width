#[derive(Debug, Clone)]
pub struct Graph {
    pub n: usize,
    pub adj: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            adj: vec![Vec::new(); n],
        }
    }

    pub fn from_edges(n: usize, edges: &[(usize, usize)]) -> Self {
        let mut g = Self::new(n);
        for &(u, v) in edges {
            if u != v && u < n && v < n {
                if !g.adj[u].contains(&v) {
                    g.adj[u].push(v);
                    g.adj[v].push(u);
                }
            }
        }
        g
    }

    pub fn edge_count(&self) -> usize {
        self.adj.iter().map(|a| a.len()).sum::<usize>() / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_loops_ignored() {
        let g = Graph::from_edges(3, &[(0, 0), (0, 1), (1, 2)]);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn duplicate_edges_ignored() {
        let g = Graph::from_edges(3, &[(0, 1), (0, 1), (1, 2)]);
        assert_eq!(g.edge_count(), 2);
    }
}
