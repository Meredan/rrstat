#[cfg(test)]
mod tests {
    use rrstat::cli::parse_event;
    use perf_event::events::Hardware;

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
}
