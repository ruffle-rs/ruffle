// Example showing how to use telemetry in Ruffle
// This demonstrates what the actual integration would look like

mod telemetry {
    include!("src/telemetry.rs");
}

fn main() {
    // Create telemetry instance when Ruffle starts
    let mut metrics = telemetry::TelemetryMetrics::new();
    
    // When a SWF file is loaded, update the telemetry with its info
    let swf_name = "file:///Users/username/Documents/my_game.swf";
    let swf_size_bytes = 204800; // Example: 200KB file
    let swf_framerate = 24.0; // Example: 24fps
    
    metrics.set_swf_name(swf_name);
    metrics.set_swf_size(swf_size_bytes / 1024); // Convert to KB
    metrics.set_swf_rate(swf_framerate);
    metrics.set_swf_start(0); // SWF starts at time 0
    
    // Write the telemetry file
    metrics.write_flm("ruffle_telemetry.flm").expect("Failed to write telemetry");
    
    println!("Created ruffle_telemetry.flm");
    println!("SWF: {}", swf_name);
    println!("Size: {} KB", swf_size_bytes / 1024);
    println!("FPS: {}", swf_framerate);
    println!("\nThis file can be opened in Adobe Scout.");
    println!("Note: Scout displays performance data from span metrics.");
    println!("As Ruffle runs SWF content, it would add timing spans for:");
    println!("  - Frame rendering (.swf.frame)");
    println!("  - ActionScript execution (.as.*)");  
    println!("  - Rendering operations (.rend.*)");
    println!("  - etc.");
}
