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

    /// Serialize to the PACE 2017 / DIMACS format expected by FlowCutter.
    /// Vertices are 1-indexed in the output.
    pub fn to_dimacs(&self) -> String {
        let m = self.edge_count();
        let mut lines = Vec::with_capacity(1 + m);
        lines.push(format!("p tw {} {}", self.n, m));
        for u in 0..self.n {
            for &v in &self.adj[u] {
                if u < v {
                    lines.push(format!("{} {}", u + 1, v + 1));
                }
            }
        }
        lines.join("\n") + "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimacs_path() {
        let g = Graph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
        assert!(g.to_dimacs().starts_with("p tw 4 3\n"));
    }

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
