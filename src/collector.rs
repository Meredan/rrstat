use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use perf_event::Counter;

use crate::types::Sample;
use crate::ringbuffer::RingBuffer;

use libc;

fn read_instruction_pointer(pid: i32) -> u64 {
    unsafe {
        // attach to the process to stop it
        if libc::ptrace(libc::PTRACE_ATTACH, pid, 0, 0) < 0 {
            return 0;
        }

        // wait for the process to stop
        let mut status = 0;
        if libc::waitpid(pid, &mut status, 0) < 0 {
            libc::ptrace(libc::PTRACE_DETACH, pid, 0, 0);
            return 0;
        }

        let mut regs: libc::user_regs_struct = std::mem::zeroed();
        let res = libc::ptrace(
            libc::PTRACE_GETREGS,
            pid,
            0,
            &mut regs as *mut _ as *mut libc::c_void,
        );

        libc::ptrace(libc::PTRACE_DETACH, pid, 0, 0);

        if res < 0 {
            return 0;
        }
        regs.rip
    }
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
