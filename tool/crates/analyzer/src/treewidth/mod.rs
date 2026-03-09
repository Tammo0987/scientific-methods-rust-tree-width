mod flowcutter;
mod native;

use crate::graph::Graph;
pub use flowcutter::TwError;

const NATIVE_THRESHOLD: usize = 500;

pub enum Solver {
    Native,
    FlowCutter,
    Auto,
}

impl Solver {
    pub fn compute(&self, g: &Graph) -> Result<u32, TwError> {
        match self {
            Solver::Native => Ok(native::compute(g)),
            Solver::FlowCutter => flowcutter::compute(g),
            Solver::Auto => {
                if g.n <= NATIVE_THRESHOLD {
                    Ok(native::compute(g))
                } else {
                    flowcutter::compute(g)
                }
            }
        }
    }
}
