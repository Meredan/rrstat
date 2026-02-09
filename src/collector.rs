use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use perf_event::Counter;

use crate::types::Sample;
use crate::ringbuffer::RingBuffer;

/// A helper to "parse" raw values into Sample struct
fn parse_sample(value: u64, pid: i32, start_time: Instant) -> Sample {
    Sample {
        value,
        pid,
        timestamp: start_time.elapsed().as_millis() as u64,
        instruction_pointer: 0,
    }
}

pub struct Collector {
    counter: Counter,
    buffer: Arc<RingBuffer>,
    running: Arc<AtomicBool>,
    pid: i32,
}

impl Collector {
    pub fn new(counter: Counter, buffer: Arc<RingBuffer>, running: Arc<AtomicBool>, pid: i32) -> Self {
        Self { counter, buffer, running, pid }
    }

   pub fn spawn(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let start_time = Instant::now();
            while self.running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                let val = self.counter.read().unwrap();
                let sample = parse_sample(val, self.pid, start_time);
                self.buffer.push(sample);
            }
        })
    }
}
