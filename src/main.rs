use anyhow::Result;
use clap::Parser;
use std::{thread, time::Duration};
use rrstat::profiler::PerfCounter;
use rrstat::cli;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use signal_hook::consts::signal::SIGINT;
use signal_hook::flag;

fn setup_ctrl_c() -> Result<Arc<AtomicBool>, anyhow::Error> {
    let term = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&term))?;
    Ok(term)
}

fn main() -> Result<()> {
    let args = cli::Args::parse();
    let event = cli::parse_event(&args.event)?;
    
    let mut perf_counter = PerfCounter::new(args.pid, event)?;
    perf_counter.enable()?;
    
    let term = setup_ctrl_c()?;
    let running = Arc::new(AtomicBool::new(true));
    let buffer = Arc::new(rrstat::ringbuffer::RingBuffer::new(1024));
    
    let collector = rrstat::collector::Collector::new(
        perf_counter.counter,
        Arc::clone(&buffer),
        Arc::clone(&running),
        args.pid,
    );
    let collector_handle = collector.spawn();
    
    let start = std::time::Instant::now();
    while !term.load(Ordering::Relaxed) && start.elapsed() < Duration::from_millis(args.duration) {
        thread::sleep(Duration::from_millis(100));
    }
    
    running.store(false, Ordering::Relaxed);
    collector_handle.join().unwrap();
    
    let samples = buffer.drain();
    println!("Collected {} samples", samples.len());
    
    Ok(())
}