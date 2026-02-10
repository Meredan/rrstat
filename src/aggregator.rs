use crate::types::{Sample, Report, FunctionStats};
use crate::symbols::SymbolResolver;
use std::collections::HashMap;

pub struct Aggregator {
    pub(crate) counts: HashMap<String, usize>,
    resolver: SymbolResolver,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            resolver: SymbolResolver::new(),
        }
    }

    pub(crate) fn fold_stack(&mut self, pid: u32, ip: u64) -> String {
        match self.resolver.resolve(pid, ip) {
            Ok(info) => {
                if let Some(name) = info.function {
                    name
                } else {
                    format!("unknown_0x{:x}", ip)
                }
            }
            Err(_) => format!("unknown_0x{:x}", ip),
        }
    }

    pub fn process_samples(&mut self, samples: Vec<Sample>) {
        for sample in samples {
            let key = self.fold_stack(sample.pid as u32, sample.instruction_pointer);
            *self.counts.entry(key).or_insert(0) += 1;
        }
    }

    pub fn generate_report(&self) -> Report {
        let total_samples = self.counts.values().sum();
        let mut stats = Vec::new();
        for (name, count) in &self.counts {
            let percentage = (*count as f64 / total_samples as f64) * 100.0;
            stats.push(FunctionStats {
                name: name.clone(),
                count: *count,
                percentage,
            });
        }
        stats.sort_by(|a, b| b.count.cmp(&a.count));
        let folded_stacks: Vec<String> = self.counts.keys().cloned().collect();
        Report {
            total_samples,
            stats,
            folded_stacks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_counts() {
            let mut agg = Aggregator::new();
            agg.counts.insert("main".to_string(), 10);
            agg.counts.insert("foo".to_string(), 5);

            let report = agg.generate_report();
            
            assert_eq!(report.total_samples, 15);
            assert_eq!(report.stats[0].name, "main");
            assert_eq!(report.stats[0].count, 10);
            assert_eq!(report.stats[1].name, "foo");
            assert_eq!(report.stats[1].count, 5);
        }

        #[test]
        fn test_unknown_folding() {
            let mut agg = Aggregator::new();
            let pid = std::process::id(); 
            let folded = agg.fold_stack(pid, 0xdeadbeef);
            assert!(folded.contains("unknown"));
            assert!(folded.contains("deadbeef"));
        }
    }
