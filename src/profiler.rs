use anyhow::Result;
use perf_event::Counter;
use perf_event::events::Event;

pub struct PerfCounter {
    pub counter: Counter,
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
}
