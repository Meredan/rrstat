use std::fmt;

/// A single sample of the measured event
#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub value: u64,
    pub pid: i32,
    pub timestamp: u64,
    pub instruction_pointer: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Debug)]
pub struct Report {
    pub total_samples: usize,
    pub stats: Vec<FunctionStats>,
    pub folded_stacks: Vec<String>,
}

impl fmt::Display for Sample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sample {{ ts: {}, pid: {}, val: {}, ip: {:#x} }}",
            self.timestamp, self.pid, self.value, self.instruction_pointer
        )
    }
}