use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// memory mapping from /proc/[pid]/maps
#[derive(Debug, Clone)]
pub struct Mapping {
    pub start: u64,
    pub end: u64,
    pub perms: String,
    pub offset: u64,
    pub pathname: String,
}

/// Finds the mapping for a specific binary or library containing the given address.
///
/// This is used to compute the "relative address" (offset) needed by addr2line.
/// Real addresses in a running process are randomized (ASLR).
///
/// Returns the mapping closest to the address that matches the given object name hint,
/// or simply the first code segment if no hint is relevant.
pub fn find_mapping_for_address(pid: u32, address: u64) -> Result<Option<Mapping>> {
    let maps_path = format!("/proc/{}/maps", pid);
    let file = File::open(&maps_path)
        .with_context(|| format!("Failed to open maps file: {}", maps_path))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if let Some(mapping) = parse_map_line(&line) {
            if address >= mapping.start && address < mapping.end {
                if mapping.perms.contains('x') {
                     return Ok(Some(mapping));
                }
            }
        }
    }

    Ok(None)
}

fn parse_map_line(line: &str) -> Option<Mapping> {
    // Format: 7f45c000-7f45e000 r-xp 00000000 08:01 123456 /path/to/file
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }

    let range_parts: Vec<&str> = parts[0].split('-').collect();
    if range_parts.len() != 2 {
        return None;
    }

    let start = u64::from_str_radix(range_parts[0], 16).ok()?;
    let end = u64::from_str_radix(range_parts[1], 16).ok()?;
    let perms = parts[1].to_string();
    let offset = u64::from_str_radix(parts[2], 16).ok()?;
    let pathname = parts[5].to_string();

    Some(Mapping {
        start,
        end,
        perms,
        offset,
        pathname,
    })
}