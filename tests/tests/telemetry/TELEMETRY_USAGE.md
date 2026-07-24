# Ruffle Telemetry Usage Guide

This guide shows how to integrate the Flash Player-compatible telemetry system into Ruffle.

## Overview

The telemetry system generates `.flm` files that are compatible with Adobe Scout, a professional Flash profiler. Files contain:
- **Session metadata**: SWF name, framerate, player version, system info
- **Performance spans**: Timing metrics for frames, rendering, ActionScript execution, etc.

## Desktop Live Streaming

Desktop builds can stream telemetry live to Scout while still buffering the full `.flm` session in memory.

- Start Ruffle with `--telemetry`
- On startup, Ruffle attempts a TCP connection to `127.0.0.1:7934`
- If connected, telemetry bytes are sent in small batches (time/size windowed) so the stream stays live without per-metric network writes
- If disconnected or unavailable, telemetry continues locally and is still written on exit via `write_flm`

Example:

```bash
cargo run --package ruffle_desktop -- --telemetry path/to/movie.swf
```

## Basic Usage

```rust
use ruffle_core::telemetry::TelemetryMetrics;

// Initialize telemetry (called once when player starts)
let mut telemetry = TelemetryMetrics::new();

// Set SWF information after loading
telemetry.set_swf_name("game.swf");
telemetry.set_swf_rate(30.0);  // 30 FPS
telemetry.set_swf_size(2048);  // 2 MB in KB

// Record performance spans during playback

// IMPORTANT: Frame Structure (Flash Player format)
// For Adobe Scout compatibility, each frame must follow this exact structure:
// 1. .enter marker (time marker - frame start boundary)
// 2. .swf.frame (VALUE metric with frame number)
// 3. Span metrics for frame work (.tlm.doplay, .rend.*, .as.*, etc.)
// 4. .exit marker (time marker - frame end boundary)

// Example: Recording a complete frame
let frame_number = 0;
let frame_start = get_current_micros();

// 1. Mark frame entry (.enter marker - critical for Scout!)
telemetry.add_frame_enter(None);  // Uses current time

// 2. Add frame marker (value metric - Flash Player format)
telemetry.add_frame(frame_number, None);

// 3. Frame work spans
// Frame timing span (like .tlm.doplay in Flash Player)
let frame_time = get_current_micros() - frame_start;
telemetry.add_span(".tlm.doplay", frame_time as i32, None);

// Example: Render timing
let render_start = get_current_micros();
// ... rendering code ...
let render_end = get_current_micros();
telemetry.add_span_timed(".rend.drawframe", render_start, render_end);

// Example: ActionScript execution
let as_start = get_current_micros();
// ... ActionScript code ...
let as_end = get_current_micros();
telemetry.add_span_timed(".as.doactions", as_start, as_end);

// 4. Mark frame exit (.exit marker - critical for Scout!)
telemetry.add_frame_exit(None);  // Uses current time

// Save telemetry file (called when closing SWF or on user request)
telemetry.write_flm("session.flm")?;
```

## Important Notes

### Frame Boundaries (.enter and .exit markers)

**CRITICAL for Adobe Scout compatibility!**

Flash Player wraps each frame with `.enter` and `.exit` time markers:
- **`.enter`** - Marks when frame execution begins (before .swf.frame)
- **`.exit`** - Marks when frame execution ends (after all frame work)

These markers use the `.time` trait class and are essential for Scout to:
- Recognize frame boundaries
- Build the timeline UI properly
- Display SWF name and frame information

Without these markers, Scout will load the file but display nothing.

```rust
// Complete frame pattern (REQUIRED for Scout):
telemetry.add_frame_enter(None);      // Frame start boundary
telemetry.add_frame(frame_num, None); // Frame number
// ... add spans for frame work ...
telemetry.add_frame_exit(None);       // Frame end boundary
```

### Frame Markers vs Frame Timing

In Flash Player telemetry:

- **`.swf.frame`** = VALUE metric containing the frame number (0, 1, 2, ...)
  - Use `telemetry.add_frame(frame_number)` 
  - This is what the analyzer uses to **count frames**
  
- **`.tlm.doplay`** = SPAN metric containing frame duration in microseconds
  - Use `telemetry.add_span(".tlm.doplay", duration, delta)`
  - This is what the analyzer uses to **calculate FPS**

### Proper Frame Recording Pattern

```rust
// At the start of each frame:
let frame_number = self.current_frame;
let frame_start_micros = get_current_micros();

// 1. Mark frame entry (.enter marker)
telemetry.add_frame_enter(None);

// 2. Add frame marker
telemetry.add_frame(frame_number, None);

// 3. Frame processing with spans
// ... rendering, ActionScript, etc. ...
// Add spans for each operation

// At the end of the frame:
let frame_end_micros = get_current_micros();
let frame_duration = (frame_end_micros - frame_start_micros) as i32;

// Add frame timing span
telemetry.add_span(".tlm.doplay", frame_duration, None);

// 4. Mark frame exit (.exit marker)
telemetry.add_frame_exit(None);
```

## Span Metric Categories

Based on Flash Player's telemetry, common span categories include:

### Frame Timing Metrics
- `.enter` - Frame entry marker (time marker - uses .time trait class)
- `.exit` - Frame exit marker (time marker - uses .time trait class)
- `.tlm.doplay` - Frame processing time span (used for FPS calculation)

**Note:** `.enter` and `.exit` are time markers, not spans. Use `add_frame_enter()` and `add_frame_exit()`.
**Note:** `.swf.frame` is a VALUE metric (frame number), not a timing span!

### Rendering Metrics
- `.rend.drawframe` - Frame rendering duration
- `.rend.screen` - Screen update (alternative frame marker)
- `.rend.displaylist` - Display list traversal
- `.rend.vector` - Vector rendering
- `.rend.bitmap` - Bitmap rendering

### ActionScript Metrics
- `.as.doactions` - AS1/AS2 bytecode execution
- `.as.verify` - AVM2 verification
- `.as.compile` - JIT compilation
- `.as.execute` - General execution

### Garbage Collection Metrics
- `.gc.mark` - GC mark phase
- `.gc.sweep` - GC sweep phase
- `.gc.collect` - Full GC cycle

### Network Metrics
- `.network.load` - Asset loading
- `.network.http` - HTTP requests

### Memory Metrics (values, not spans)
- `.mem.total` - Total system memory
- `.mem.used` - Used memory
- `.mem.managed.used` - GC-managed memory

## Integration Points

### 1. Player Initialization
```rust
// In Player::new() or similar
self.telemetry = Some(TelemetryMetrics::new());
```

### 2. SWF Loading
```rust
// After loading SWF
if let Some(telemetry) = &mut self.telemetry {
    telemetry.set_swf_name(&swf_url);
    telemetry.set_swf_rate(swf.frame_rate);
    telemetry.set_swf_size(swf_data.len() / 1024);
}
```

### 3. Frame Loop
```rust
// In Player::run_frame() or equivalent
let frame_start = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_micros();

// CRITICAL: Add .enter marker FIRST (for Scout compatibility)
if let Some(telemetry) = &mut self.telemetry {
    telemetry.add_frame_enter(None);
}

// Add frame marker (value metric with frame number)
if let Some(telemetry) = &mut self.telemetry {
    telemetry.add_frame(self.current_frame, None);
}

// ... process frame (rendering, ActionScript, etc.) ...

// At frame end, add frame timing span
if let Some(telemetry) = &mut self.telemetry {
    let frame_end = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    telemetry.add_span_timed(".tlm.doplay", frame_start, frame_end);
}

// CRITICAL: Add .exit marker LAST (for Scout compatibility)
if let Some(telemetry) = &mut self.telemetry {
    telemetry.add_frame_exit(None);
}
```

### 4. Rendering
```rust
// In RenderBackend::render() or similar
let render_start = get_micros();
// ... rendering ...
context.telemetry.add_span_timed(".rend.drawframe", render_start, get_micros());
```

### 5. ActionScript Execution
```rust
// In AVM execution
let exec_start = get_micros();
// ... run bytecode ...
context.telemetry.add_span_timed(".as.doactions", exec_start, get_micros());
```

### 6. Saving Output
```rust
// On player shutdown or user request
if let Some(telemetry) = &self.telemetry {
    telemetry.write_flm("ruffle_session.flm")?;
}
```

## Analyzing Results

### With Python Script
```bash
cd core
python3 telemetry.py session.flm
```

Output shows:
- Frame count and FPS
- Time breakdown by category (Player, Rendering, ActionScript, etc.)
- Memory usage
- Performance timeline

### With Adobe Scout

1. Open Adobe Scout
2. File → Open Session
3. Select the `.flm` file
4. View performance timeline, frame breakdowns, and detailed metrics

### Command-line Options
```bash
# Show frame-by-frame breakdown
python3 telemetry.py -f session.flm

# Show detailed metrics
python3 telemetry.py -s session.flm

# Show memory stats
python3 telemetry.py -m session.flm

# Filter by load percentage
python3 telemetry.py -l 50 session.flm  # Only frames >50% load

# Analyze specific frame range
python3 telemetry.py --range 10:20 session.flm  # Frames 10-20
```

## Performance Considerations

- Telemetry has minimal overhead (~0.1% typically)
- Only enable when profiling is needed
- Consider using a compile-time feature flag:

```rust
#[cfg(feature = "telemetry")]
pub telemetry: Option<TelemetryMetrics>,
```

## File Format

The `.flm` format uses AMF3 encoding with trait-based sealed properties:

1. Session metadata: Objects with class `.value`, properties `name` and `value`
2. Performance spans: Objects with class `.span`, properties `name`, `span`, `delta`

Format is compatible with Flash Player 11+ telemetry (version "3,2").
