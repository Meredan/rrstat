use clap::Parser;
use perf_event::events::{Hardware, Event};
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
        _ => bail!("Unknown event: {}", event_name),
    }
}
