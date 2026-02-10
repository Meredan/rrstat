use crate::types::Report;

pub fn print_summary(report: &Report) {
    println!("\n{:=^60}", " PROFILER SUMMARY ");
    println!("Total Samples: {}", report.total_samples);
    println!("{:-^60}", "");
    
    println!("{:<40} | {:>8} | {:>8}", "Function / Context", "Samples", "%");
    println!("{:-^60}", "");

    for stat in &report.stats {
        let display_name = if stat.name.len() > 38 {
            format!("{}..", &stat.name[..36])
        } else {
            stat.name.clone()
        };
        
        println!("{:<40} | {:>8} | {:>8.2}%", 
            display_name, 
            stat.count, 
            stat.percentage
        );
    }
    println!("{:=^60}\n", "");
}
