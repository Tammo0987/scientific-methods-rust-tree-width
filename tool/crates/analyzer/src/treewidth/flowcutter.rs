use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::graph::Graph;

#[derive(Debug)]
pub enum TwError {
    SolverNotFound(String),
    SolverFailed(String),
    Timeout,
}

impl std::fmt::Display for TwError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TwError::SolverNotFound(msg) => write!(f, "solver not found: {msg}"),
            TwError::SolverFailed(msg) => write!(f, "solver failed: {msg}"),
            TwError::Timeout => write!(f, "solver timed out"),
        }
    }
}

impl std::error::Error for TwError {}

pub fn compute(g: &Graph) -> Result<u32, TwError> {
    if g.n <= 1 || g.edge_count() == 0 {
        return Ok(0);
    }

    let bin = env::var("FLOW_CUTTER_BIN").unwrap_or_else(|_| "flow_cutter_pace17".to_string());

    let timeout_secs = env::var("FLOW_CUTTER_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);

    let mut tmp = tempfile::NamedTempFile::new()
        .map_err(|e| TwError::SolverFailed(format!("tempfile: {e}")))?;
    tmp.write_all(g.to_dimacs().as_bytes())
        .map_err(|e| TwError::SolverFailed(format!("write tempfile: {e}")))?;
    tmp.flush()
        .map_err(|e| TwError::SolverFailed(format!("flush tempfile: {e}")))?;

    // Use `timeout` to send SIGTERM after the time limit. FlowCutter prints
    // its best decomposition upon receiving SIGTERM and exits cleanly.
    let output = Command::new("timeout")
        .arg(timeout_secs.to_string())
        .arg(&bin)
        .arg(tmp.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| TwError::SolverNotFound(format!("could not run timeout/'{bin}': {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    // Empty stdout with a failed exit means the process was killed before
    // producing any output (e.g. timeout before first decomposition).
    if stdout.is_empty() && !output.status.success() {
        return Err(TwError::Timeout);
    }

    parse_output(&stdout)
}

/// Parse treewidth from FlowCutter's PACE tree decomposition output.
///
/// FlowCutter deviates from the PACE spec on the `s` header line field order,
/// so we compute treewidth directly from the bag lines:
///   treewidth = max(|bag|) - 1
fn parse_output(output: &str) -> Result<u32, TwError> {
    let mut max_bag_size: Option<usize> = None;

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Bag line: "b <bag_id> <v1> <v2> ..."
        if parts.first() == Some(&"b") && parts.len() >= 2 {
            let bag_size = parts.len() - 2; // subtract "b" and bag_id
            max_bag_size = Some(max_bag_size.unwrap_or(0).max(bag_size));
        }
    }

    match max_bag_size {
        Some(0) | None => Err(TwError::SolverFailed(format!(
            "no bag lines found in output: {:?}",
            output.chars().take(300).collect::<String>()
        ))),
        Some(s) => Ok((s - 1) as u32),
    }
}
