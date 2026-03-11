use std::env;
use std::fs;
use std::io::ErrorKind;
use std::io::Write;
#[cfg(not(has_flowcutter_ffi))]
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

pub fn ensure_solver_available() -> Result<(), TwError> {
    #[cfg(has_flowcutter_ffi)]
    {
        // In-process C++ FFI is available, no external binary required.
        return Ok(());
    }

    #[cfg(not(has_flowcutter_ffi))]
    {
        let bin = env::var("FLOW_CUTTER_BIN").unwrap_or_else(|_| "flow_cutter_pace17".to_string());
        if bin.trim().is_empty() {
            return Err(TwError::SolverNotFound(
                "FLOW_CUTTER_BIN is empty; set it to the FlowCutter executable path".to_string(),
            ));
        }

        if binary_exists(&bin) {
            Ok(())
        } else {
            Err(TwError::SolverNotFound(format!(
                "could not find '{bin}' (set FLOW_CUTTER_BIN=/absolute/path/to/flow_cutter_pace17)"
            )))
        }
    }
}

pub fn compute(g: &Graph) -> Result<u32, TwError> {
    if g.n <= 1 || g.edge_count() == 0 {
        return Ok(0);
    }

    #[cfg(has_flowcutter_ffi)]
    {
        if let Some(tw) = compute_with_ffi(g)? {
            return Ok(tw);
        }
    }

    compute_with_subprocess(g)
}

#[cfg(has_flowcutter_ffi)]
fn compute_with_ffi(g: &Graph) -> Result<Option<u32>, TwError> {
    if g.n > u32::MAX as usize {
        return Ok(None);
    }

    let m = g.edge_count();
    if m > u32::MAX as usize {
        return Ok(None);
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
        0 => Ok(Some(out_tw)),
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

fn compute_with_subprocess(g: &Graph) -> Result<u32, TwError> {
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

    let solver_stdout = tempfile::NamedTempFile::new()
        .map_err(|e| TwError::SolverFailed(format!("stdout tempfile: {e}")))?;
    let stdout_writer = solver_stdout
        .reopen()
        .map_err(|e| TwError::SolverFailed(format!("reopen stdout tempfile: {e}")))?;

    // Run the external C binary directly. Timeout is enforced in Rust so we do
    // not depend on GNU `timeout` (missing on macOS by default).
    let mut child = Command::new(&bin)
        .arg(tmp.path())
        .stdout(Stdio::from(stdout_writer))
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => TwError::SolverNotFound(format!("could not run '{bin}': {e}")),
            _ => TwError::SolverFailed(format!("could not spawn '{bin}': {e}")),
        })?;

    let timed_out = wait_with_timeout(&mut child, Duration::from_secs(timeout_secs))
        .map_err(|e| TwError::SolverFailed(format!("failed waiting for '{bin}': {e}")))?;

    let stdout = fs::read_to_string(solver_stdout.path())
        .map_err(|e| TwError::SolverFailed(format!("read solver output: {e}")))?;

    // Empty stdout with a failed exit means the process was killed before
    // producing any output (e.g. timeout before first decomposition).
    if stdout.trim().is_empty() && timed_out {
        return Err(TwError::Timeout);
    }

    parse_output(&stdout)
}

#[cfg(not(has_flowcutter_ffi))]
fn binary_exists(bin: &str) -> bool {
    let path = Path::new(bin);
    if path.is_absolute() || bin.contains('/') {
        return path.is_file();
    }

    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(bin).is_file()))
        .unwrap_or(false)
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> std::io::Result<bool> {
    let start = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return Ok(false);
        }

        if start.elapsed() >= timeout {
            terminate_soft(child)?;

            let grace_start = Instant::now();
            while grace_start.elapsed() < Duration::from_millis(200) {
                if child.try_wait()?.is_some() {
                    return Ok(true);
                }
                thread::sleep(Duration::from_millis(10));
            }

            let _ = child.kill();
            let _ = child.wait();
            return Ok(true);
        }

        thread::sleep(Duration::from_millis(20));
    }
}

#[cfg(unix)]
fn terminate_soft(child: &mut Child) -> std::io::Result<()> {
    let pid = child.id() as i32;
    let rc = unsafe { libc::kill(pid, libc::SIGTERM) };
    if rc == 0 {
        Ok(())
    } else {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[cfg(not(unix))]
fn terminate_soft(child: &mut Child) -> std::io::Result<()> {
    match child.kill() {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::InvalidInput => Ok(()),
        Err(e) => Err(e),
    }
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
