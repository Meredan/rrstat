use std::collections::VecDeque;
use std::sync::Mutex;
use crate::types::Sample;

/// A thread-safe ring buffer with a fixed capacity.
pub struct RingBuffer {
    data: Mutex<VecDeque<Sample>>,
    capacity: usize,
}

impl RingBuffer {
    /// Creates a new RingBuffer with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    pub fn is_empty(&self) -> bool {
        let data = self.data.lock().unwrap();
        data.is_empty()
    }
    
    pub fn push(&self, sample: Sample) {
        let mut data = self.data.lock().unwrap();
        if data.len() == self.capacity {
            data.pop_front();
        }
        data.push_back(sample);
    }
    
    pub fn drain(&self) -> Vec<Sample> {
        let mut data = self.data.lock().unwrap();
        return data.drain(..).collect();
    }
}
