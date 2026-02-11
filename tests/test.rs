#[cfg(test)]
mod tests {
    use rrstat::cli::parse_event;
    use rrstat::ringbuffer::RingBuffer;
    use rrstat::types::Sample;
    use rrstat::symbols::SymbolResolver;
    use std::process::Command;
    use anyhow::Result;


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
        let rb = RingBuffer::new(2); 
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

    #[test]
    fn test_resolve_symbol() -> Result<()> {
        let source = r#"
            #include <stdio.h>
            #include <unistd.h>
            void target_function() {
                printf("Hello\n");
                sleep(1);
            }
            int main() {
                target_function();
                return 0;
            }
        "#;
        std::fs::write("dummy_target.c", source)?;
        
        let status = Command::new("gcc")
            .args(&["-g", "dummy_target.c", "-o", "dummy_target"])
            .status()?;
        assert!(status.success());
        
        let mut child = Command::new("./dummy_target").spawn()?;
        let pid = child.id();
        
        let output = Command::new("nm")
            .arg("dummy_target")
            .output()?;
        let output_str = String::from_utf8(output.stdout)?;
        
        let mut addr = 0;
        for line in output_str.lines() {
            if line.contains("target_function") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                addr = u64::from_str_radix(parts[0], 16)?;
                break;
            }
        }
        assert!(addr != 0, "Could not find target_function address");
        
        let mut load_base = 0;
        let maps_path = format!("/proc/{}/maps", pid);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let maps_content = std::fs::read_to_string(maps_path)?;
        for line in maps_content.lines() {
            if line.contains("dummy_target") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let range: Vec<&str> = parts[0].split('-').collect();
                let offset = u64::from_str_radix(parts[2], 16)?;
                if offset == 0 {
                    load_base = u64::from_str_radix(range[0], 16)?;
                    break;
                }
            }
        }
        assert!(load_base != 0, "Could not find load base for dummy_target");
        let absolute_addr = load_base + addr;

        let mut resolver = SymbolResolver::new();
        let info = resolver.resolve(pid, absolute_addr);
        
        let _ = child.kill();
        let _ = std::fs::remove_file("dummy_target");
        let _ = std::fs::remove_file("dummy_target.c");
        match info {
            Ok(sym) => {
                println!("Resolved: {:?}", sym);
                assert!(sym.function.unwrap().contains("target_function"));
            }
            Err(e) => {
                eprintln!("Resolution failed: {:?}", e);
                panic!("Resolution failed");
            }
        }
        Ok(())
    }

    #[test]
    fn test_end_to_end_report_generation() -> Result<()> {
        use rrstat::aggregator::Aggregator;
        use rrstat::types::Sample;
        use std::time::Duration;

        let source = r#"
            #include <stdio.h>
            #include <unistd.h>
            void func_a() { usleep(100); }
            void func_b() { usleep(100); }
            int main() {
                while(1) { func_a(); func_b(); }
                return 0;
            }
        "#;
        std::fs::write("dummy_agg.c", source)?;
        let status = Command::new("gcc")
            .args(&["-g", "dummy_agg.c", "-o", "dummy_agg"])
            .status()?;
        assert!(status.success());
        
        let mut child = Command::new("./dummy_agg").spawn()?;
        let pid = child.id();
        std::thread::sleep(Duration::from_millis(150));
   
        let mut addr_a = 0;
        let mut addr_b = 0;
        
        // Use nm to find offsets
        let output = Command::new("nm").arg("dummy_agg").output()?;
        let output_str = String::from_utf8(output.stdout)?;
        for line in output_str.lines() {
            if line.contains("func_a") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                addr_a = u64::from_str_radix(parts[0], 16)?;
            } else if line.contains("func_b") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                addr_b = u64::from_str_radix(parts[0], 16)?;
            }
        }
        
        // Adjust for PIE load base
        let mut load_base = 0;
        let maps_path = format!("/proc/{}/maps", pid);
        let maps_content = std::fs::read_to_string(maps_path)?;
        for line in maps_content.lines() {
            if line.contains("dummy_agg") {
                 let parts: Vec<&str> = line.split_whitespace().collect();
                 let range: Vec<&str> = parts[0].split('-').collect();
                 let offset = u64::from_str_radix(parts[2], 16).unwrap_or(1);
                 if offset == 0 {
                     load_base = u64::from_str_radix(range[0], 16)?;
                     break;
                 }
            }
        }
        
        // Create fake samples
        let s1 = Sample { pid: pid as i32, instruction_pointer: load_base + addr_a, value: 1, timestamp: 100 };
        let s2 = Sample { pid: pid as i32, instruction_pointer: load_base + addr_b, value: 1, timestamp: 200 };
        let s3 = Sample { pid: pid as i32, instruction_pointer: load_base + addr_a, value: 1, timestamp: 300 };

        let mut agg = Aggregator::new();
        agg.process_samples(vec![s1, s2, s3]);
        
        let report = agg.generate_report();
        println!("Report: {:?}", report);

        // Verification
        assert_eq!(report.total_samples, 3);
        let has_func_a = report.stats.iter().any(|s| s.name.contains("func_a") && s.count == 2);
        let has_func_b = report.stats.iter().any(|s| s.name.contains("func_b") && s.count == 1);
        
        // Cleanup
        let _ = child.kill();
        let _ = std::fs::remove_file("dummy_agg");
        let _ = std::fs::remove_file("dummy_agg.c");

        assert!(has_func_a, "Report missing func_a with count 2");
        assert!(has_func_b, "Report missing func_b with count 1");

        Ok(())
    }

    #[test]
    fn test_real_collector() -> Result<()> {
        use rrstat::collector::Collector;
        use rrstat::profiler::PerfCounter;
        use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
        use std::time::Duration;

        // 1. Compile a CPU-bound target
        let source = r#"
            #include <math.h>
            #include <stdio.h>
            int main() {
                double x = 0;
                while(1) { x += sin(x); }
                return 0;
            }
        "#;
        std::fs::write("cpu_burner.c", source)?;
        let status = Command::new("gcc")
            .args(&["-g", "cpu_burner.c", "-o", "cpu_burner", "-lm"]) // Link math lib
            .status()?;
        assert!(status.success());

        let mut child = Command::new("./cpu_burner").spawn()?;
        let pid = child.id() as i32;
        
        let event = parse_event("cpu-cycles")?;
        let mut pc = PerfCounter::new(pid, event)?;
        pc.enable()?;

        let buffer = Arc::new(RingBuffer::new(1024));
        let running = Arc::new(AtomicBool::new(true));
        
        let collector = Collector::new(pc.counter, Arc::clone(&buffer), Arc::clone(&running), pid);
        let handle = collector.spawn();

        std::thread::sleep(Duration::from_millis(500));
        
        running.store(false, Ordering::Relaxed);
        handle.join().unwrap();
        let samples = buffer.drain();
        
        let _ = child.kill();
        let _ = std::fs::remove_file("cpu_burner");
        let _ = std::fs::remove_file("cpu_burner.c");

        println!("Collected {} samples", samples.len());
        assert!(samples.len() > 0, "No samples collected!");
        
   
        let non_zero_ips = samples.iter().filter(|s| s.instruction_pointer != 0).count();
        println!("Non-zero IPs: {}", non_zero_ips);
        assert!(non_zero_ips > 0, "All samples had 0 IP (ptrace failed?)");

        use rrstat::aggregator::Aggregator;
        use rrstat::symbols::SymbolResolver;
        let mut agg = Aggregator::new();
        agg.process_samples(samples);
        let report = agg.generate_report();
        
        println!("Collected Report:");
        for stat in &report.stats {
            println!("  Function: {}, Count: {}", stat.name, stat.count);
        }

        Ok(())
    }
}
