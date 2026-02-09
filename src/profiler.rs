use anyhow::{Context, Result};
use perf_event::{Builder, Counter, Group};
use perf_event::events::{Hardware, Software, Event};
use std::time::Duration;

pub struct PerfCounter {
    counter: Counter,
}

impl PerfCounter {

    pub fn new(pid: i32, event: Event) -> Result<Self> {
        let counter = perf_event::Builder::new()
            .kind(event)
            .observe_pid(pid)
            .build()?;
            
        Ok(Self { counter })
    }

    pub fn enable(&mut self) -> Result<()> {
        self.counter.enable()?;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.counter.disable()?;
        Ok(())
    }

    pub fn read(&mut self) -> Result<u64> {
        return Ok(self.counter.read()?);
    }
}

pub fn start_sampling(frequency: u64) {
    println!("Sampling at {} Hz", frequency);
}
