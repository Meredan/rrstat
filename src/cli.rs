use clap::Parser;
use perf_event::events::{Hardware, Event, Software};
use anyhow::{bail, Result};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub pid: i32,

    #[arg(short, long, default_value = "cpu-cycles")]
    pub event: String,

    #[arg(short, long, default_value = "1000")]
    pub duration: u64,
}

pub fn parse_event(event_name: &str) -> Result<Event> {
    match event_name {
        "cpu-cycles" => Ok(Event::Hardware(Hardware::CPU_CYCLES)),
        "instructions" => Ok(Event::Hardware(Hardware::INSTRUCTIONS)),
        "cache-references" => Ok(Event::Hardware(Hardware::CACHE_REFERENCES)),
        "cache-misses" => Ok(Event::Hardware(Hardware::CACHE_MISSES)),
        "task-clock" => Ok(Event::Software(Software::TASK_CLOCK)),
        "cpu-clock" | "wait-time" => Ok(Event::Software(Software::CPU_CLOCK)),
        "context-switches" => Ok(Event::Software(Software::CONTEXT_SWITCHES)),
        "page-faults" => Ok(Event::Software(Software::PAGE_FAULTS)),
        _ => bail!("Unknown event: {}", event_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
