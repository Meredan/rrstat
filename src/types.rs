use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Sample {
    pub value: u64,
    pub pid: i32,
    pub timestamp: u64,
    pub instruction_pointer: u64,
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
