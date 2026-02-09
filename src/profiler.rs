use anyhow::Result;
use perf_event::Counter;
use perf_event::events::Event;

pub struct PerfCounter {
    pub counter: Counter,
}

//setup what events we want to measure, which PID to measure it on, and then create a counter
impl PerfCounter {
    pub fn new(pid: i32, event: Event) -> Result<Self> {
        let counter = perf_event::Builder::new()
            .kind(event)
            .observe_pid(pid)
            //the moment when library makes a syscall perf_event_open
            .build()?;
            
        Ok(Self { counter })
    }

    pub fn enable(&mut self) -> Result<()> {
        self.counter.enable()?;
        Ok(())
    }
}
