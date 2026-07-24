mod telemetry {
    include!("src/telemetry.rs");
}

fn main() {
    let mut metrics = telemetry::TelemetryMetrics::new();
    
    // Simulate loading a real SWF with known parameters
    metrics.set_swf_name("game.swf");
    metrics.set_swf_rate(24.0);  // 24fps
    metrics.set_swf_start(1000);  // Started at 1 second
    
    metrics.write_flm("test_complete.flm").expect("Failed to write");
    println!("Created test_complete.flm with complete SWF info");
}
