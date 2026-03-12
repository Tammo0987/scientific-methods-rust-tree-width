use crate::graph::Graph;

#[derive(Debug)]
pub enum TwError {
    OracleUnavailable(String),
    SolverFailed(String),
}

impl std::fmt::Display for TwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwError::OracleUnavailable(msg) => write!(f, "oracle unavailable: {msg}"),
            TwError::SolverFailed(msg) => write!(f, "solver failed: {msg}"),
        }
    }
}

impl std::error::Error for TwError {}

#[cfg(has_flowcutter_ffi)]
unsafe extern "C" {
    fn flowcutter_compute_treewidth_from_edges(
        node_count: u32,
        edge_count: u32,
        edge_u: *const u32,
        edge_v: *const u32,
        out_treewidth: *mut u32,
    ) -> i32;
}

pub fn ensure_oracle_available() -> Result<(), TwError> {
    if cfg!(has_flowcutter_ffi) {
        Ok(())
    } else {
        Err(TwError::OracleUnavailable(
            "FlowCutter FFI is not built; initialise the flow-cutter-pace17 submodule before compiling analyzer"
                .to_string(),
        ))
    }
}

pub fn compute_oracle(g: &Graph) -> Result<u32, TwError> {
    ensure_oracle_available()?;
    if g.n <= 1 || g.edge_count() == 0 {
        return Ok(0);
    }
    call_ffi(g)
}

#[cfg(has_flowcutter_ffi)]
fn call_ffi(g: &Graph) -> Result<u32, TwError> {
    if g.n > u32::MAX as usize {
        return Err(TwError::SolverFailed(
            "graph too large for flowcutter ffi node count".to_string(),
        ));
    }

    let m = g.edge_count();
    if m > u32::MAX as usize {
        return Err(TwError::SolverFailed(
            "graph too large for flowcutter ffi edge count".to_string(),
        ));
    }

    let mut edge_u = Vec::with_capacity(m);
    let mut edge_v = Vec::with_capacity(m);
    for u in 0..g.n {
        for &v in &g.adj[u] {
            if u < v {
                edge_u.push(u as u32);
                edge_v.push(v as u32);
            }
        }
    }

    let mut out_tw = 0u32;
    let rc = unsafe {
        flowcutter_compute_treewidth_from_edges(
            g.n as u32,
            edge_u.len() as u32,
            edge_u.as_ptr(),
            edge_v.as_ptr(),
            &mut out_tw as *mut u32,
        )
    };

    match rc {
        0 => Ok(out_tw),
        -1 => Err(TwError::SolverFailed(
            "flowcutter ffi failed: null output pointer".to_string(),
        )),
        -2 => Err(TwError::SolverFailed(
            "flowcutter ffi failed: null edge arrays".to_string(),
        )),
        -3 => Err(TwError::SolverFailed(
            "flowcutter ffi failed: edge endpoint out of range".to_string(),
        )),
        -4 => Err(TwError::SolverFailed(
            "flowcutter ffi failed with C++ exception".to_string(),
        )),
        other => Err(TwError::SolverFailed(format!(
            "flowcutter ffi failed with status code {other}"
        ))),
    }
}

#[cfg(not(has_flowcutter_ffi))]
fn call_ffi(_: &Graph) -> Result<u32, TwError> {
    // ensure_oracle_available() returns Err before we ever reach here;
    // this stub exists only to satisfy the compiler in non-FFI builds.
    unreachable!("guarded by ensure_oracle_available")
}
