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
        
        // Compile with debug info (-g)
        let status = Command::new("gcc")
            .args(&["-g", "dummy_target.c", "-o", "dummy_target"])
            .status()?;
        assert!(status.success());
        
        // Run it in background
        let mut child = Command::new("./dummy_target").spawn()?;
        let pid = child.id();
        
        //  Find address of target_function (using nm)
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
        
        let mut absolute_addr = 0;
        let maps_path = format!("/proc/{}/maps", pid);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let maps_content = std::fs::read_to_string(maps_path)?;
        for line in maps_content.lines() {
            if line.contains("dummy_target") && line.contains("r-x") {
                let start_hex = line.split('-').next().unwrap();
                let start_addr = u64::from_str_radix(start_hex, 16)?;
                absolute_addr = start_addr + addr;
                break;
            }
        }
        assert!(absolute_addr != 0, "Could not find base address for dummy_target");

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
}
