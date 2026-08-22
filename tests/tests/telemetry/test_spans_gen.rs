// Quick test program to generate a .flm file with spans for testing

use std::time::{SystemTime, UNIX_EPOCH};

// Include the telemetry module directly
include!("src/telemetry.rs");

fn main() {
    let mut telemetry = TelemetryMetrics::new();
    
    // Set SWF info
    telemetry.set_swf_name("test_with_spans.swf");
    telemetry.set_swf_rate(30.0);
    telemetry.set_swf_size(150);
    
    // Add example span metrics simulating a few frames of activity
    let base_time = 0;
    
    // Frame 1
    telemetry.add_span(".swf.frame", 33333, Some(base_time));  // 33ms frame
    telemetry.add_span(".rend.drawframe", 15000, Some(base_time + 1000));  // 15ms render
    telemetry.add_span(".as.doactions", 10000, Some(base_time + 16000));  // 10ms AS execution
    telemetry.add_span(".gc.mark", 2000, Some(base_time + 26000));  // 2ms GC mark phase
    
    // Frame 2
    let frame2 = base_time + 33333;
    telemetry.add_span(".swf.frame", 33333, Some(frame2));
    telemetry.add_span(".rend.drawframe", 12000, Some(frame2 + 1000));
    telemetry.add_span(".as.doactions", 15000, Some(frame2 + 13000));
    
    // Frame 3
    let frame3 = frame2 + 33333;
    telemetry.add_span(".swf.frame", 33333, Some(frame3));
    telemetry.add_span(".rend.drawframe", 14000, Some(frame3 + 1000));
    telemetry.add_span(".as.doactions", 12000, Some(frame3 + 15000));
    
    // Write the file
    telemetry.write_flm("test_with_spans.flm").expect("Failed to write file");
    
    println!("Generated test_with_spans.flm with {} metrics and {} spans", 
             telemetry.metrics.len(), telemetry.spans.len());
    println!("\nMetrics:");
    for (name, _) in &telemetry.metrics {
        println!("  {}", name);
    }
    println!("\nSpans:");
    for span in &telemetry.spans {
        println!("  {} (span={}us, delta={}us)", span.name, span.span, span.delta);
    }
}
