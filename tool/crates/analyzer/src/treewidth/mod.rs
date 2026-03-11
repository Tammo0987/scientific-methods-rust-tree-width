mod flowcutter;
mod native;

use crate::graph::Graph;
pub use flowcutter::ensure_oracle_available;
pub use flowcutter::TwError;

pub fn compute(g: &Graph) -> u32 {
    native::compute(g)
}

pub fn compute_oracle(g: &Graph) -> Result<u32, TwError> {
    flowcutter::compute_oracle(g)
}
