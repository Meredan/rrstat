use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use perf_event::Counter;

use crate::types::Sample;
use crate::ringbuffer::RingBuffer;

fn read_instruction_pointer(pid: i32) -> u64 {
    let path = format!("/proc/{}/stat", pid);
    if let Ok(content) = std::fs::read_to_string(&path) {
        // Fields in /proc/[pid]/stat are space-separated after comm (which may contain spaces)
        // Find the closing ')' of comm field, then parse remaining fields
        if let Some(pos) = content.rfind(')') {
            let fields: Vec<&str> = content[pos + 2..].split_whitespace().collect();
            // kstkeip is field 30 in stat (1-indexed), which is index 27 after comm
            if fields.len() > 27 {
                return fields[27].parse::<u64>().unwrap_or(0);
            }
        }
    }
    0
}

fn parse_sample(value: u64, pid: i32, start_time: Instant, ip: u64) -> Sample {
    Sample {
        value,
        pid,
        timestamp: start_time.elapsed().as_millis() as u64,
        instruction_pointer: ip,
    }
}

pub struct Collector {
    counter: Counter,
    buffer: Arc<RingBuffer>,
    running: Arc<AtomicBool>,
    pid: i32,
}

/// Collector polls the counter and pushes samples to the ring buffer
impl Collector {
    pub fn new(counter: Counter, buffer: Arc<RingBuffer>, running: Arc<AtomicBool>, pid: i32) -> Self {
        Self { counter, buffer, running, pid }
    }

   /// New thread collects samples, while main can handle Ctrl+C -> it's unblocked
    pub fn spawn(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let start_time = Instant::now();
            while self.running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(100));
                let val = self.counter.read().unwrap();
                let ip = read_instruction_pointer(self.pid);
                let sample = parse_sample(val, self.pid, start_time, ip);
                self.buffer.push(sample);
            }
        })
    }
}
