mod flowcutter;
mod native;

use crate::graph::Graph;
pub use flowcutter::ensure_oracle_available;
pub use flowcutter::TwError;

pub enum Solver {
    Native,
}

impl Solver {
    pub fn compute(&self, g: &Graph) -> u32 {
        match self {
            Solver::Native => native::compute(g),
        }
    }
}

pub fn compute_oracle(g: &Graph) -> Result<u32, TwError> {
    flowcutter::compute_oracle(g)
}
