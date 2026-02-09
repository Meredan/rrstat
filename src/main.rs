use anyhow::Result;
use clap::Parser;
use std::{thread, time::Duration};
use rrstat::profiler::PerfCounter;
use rrstat::cli;

fn main() -> Result<()> {
    let args = cli::Args::parse();
    let event = cli::parse_event(&args.event)?;
    let mut counter = PerfCounter::new(args.pid, event)?;
    counter.enable()?;
    thread::sleep(Duration::from_millis(args.duration));
    counter.disable()?;
    let count = counter.read()?;
    println!("Count: {}", count);
    Ok(())
}