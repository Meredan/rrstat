#[cfg(test)]
mod tests {
    use rrstat::cli::parse_event;
    use rrstat::ringbuffer::RingBuffer;
    use rrstat::types::Sample;

    #[test]
    fn test_parse_cpu_cycles() {
        let event = parse_event("cpu-cycles");
        assert!(event.is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        let event = parse_event("invalid-event-name");
        assert!(event.is_err());
    }

    #[test]
    fn test_ring_buffer_overwrite() {
        let rb = RingBuffer::new(2); // Tiny capacity
        let s1 = Sample { value: 10, pid: 1, timestamp: 100, instruction_pointer: 0 };
        let s2 = Sample { value: 20, pid: 1, timestamp: 200, instruction_pointer: 0 };
        let s3 = Sample { value: 30, pid: 1, timestamp: 300, instruction_pointer: 0 };
        rb.push(s1);
        rb.push(s2);        
        rb.push(s3);
        let samples = rb.drain();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].value, 20);
        assert_eq!(samples[1].value, 30);
    }
}
