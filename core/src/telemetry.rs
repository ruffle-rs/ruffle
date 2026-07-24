use flash_lso::types::Value as AmfValue;
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use web_time::Instant;

#[cfg(not(target_family = "wasm"))]
use std::net::TcpStream;

/// Returns the current process resident set size (RSS) in kilobytes.
/// Falls back to 0 if the platform is unsupported or the call fails.
pub fn get_process_rss_kb() -> i32 {
    #[cfg(target_os = "macos")]
    {
        macos_rss_kb()
    }
    #[cfg(target_os = "linux")]
    {
        linux_rss_kb()
    }
    #[cfg(windows)]
    {
        windows_rss_kb()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
    {
        0
    }
}

#[cfg(target_os = "macos")]
fn macos_rss_kb() -> i32 {
    use std::mem;
    #[repr(C)]
    struct MachTaskBasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [u32; 2],   // time_value_t
        system_time: [u32; 2], // time_value_t
        policy: i32,
        suspend_count: i32,
    }
    unsafe extern "C" {
        fn mach_task_self() -> u32;
        fn task_info(
            target_task: u32,
            flavor: u32,
            task_info_out: *mut MachTaskBasicInfo,
            task_info_count: *mut u32,
        ) -> i32;
    }
    const MACH_TASK_BASIC_INFO: u32 = 20;
    const MACH_TASK_BASIC_INFO_COUNT: u32 =
        (mem::size_of::<MachTaskBasicInfo>() / mem::size_of::<u32>()) as u32;
    unsafe {
        let mut info: MachTaskBasicInfo = mem::zeroed();
        let mut count = MACH_TASK_BASIC_INFO_COUNT;
        let kr = task_info(
            mach_task_self(),
            MACH_TASK_BASIC_INFO,
            &mut info as *mut _,
            &mut count,
        );
        if kr == 0 {
            (info.resident_size / 1024) as i32
        } else {
            0
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_rss_kb() -> i32 {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if let Some(val) = line.strip_prefix("VmRSS:") {
                let val = val.trim().trim_end_matches(" kB").trim();
                return val.parse().unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(windows)]
fn windows_rss_kb() -> i32 {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    let mut counters = PROCESS_MEMORY_COUNTERS::default();
    let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    unsafe {
        if GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, size) != 0 {
            (counters.WorkingSetSize / 1024) as i32
        } else {
            0
        }
    }
}

/// RAII guard for timing spans - automatically records span duration on drop
pub struct SpanGuard<'a> {
    telemetry: &'a mut TelemetryMetrics,
    name: String,
    start_micros: u128,
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        let end_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();

        self.telemetry
            .add_span_timed(&self.name, self.start_micros, end_micros);
    }
}

/// Telemetry metrics with streaming buffer (writes directly, never updates)
pub struct TelemetryMetrics {
    enabled: bool,
    buffer: Vec<u8>,           // AMF3-encoded telemetry data
    string_table: Vec<String>, // AMF3 string reference table
    trait_table: Vec<String>,  // AMF3 trait reference table (class names)
    #[allow(dead_code)] // only used for testing, but useful to have for calculating deltas
    session_start_micros: u128,
    last_event_micros: u128, // Track time of last event for relative deltas
    // SWF info - stored until set_swf_start() is called
    swf_name: String,
    swf_rate_micros: i32, // Microseconds per frame
    swf_size_kb: i32,
    swf_width: i32,          // Width in pixels
    swf_height: i32,         // Height in pixels
    swf_vm: i32,             // ActionScript VM version (1 = AVM1, 2 = AVM2)
    swf_start_written: bool, // Has .swf.start been written yet?
    live_stream_offset: usize,
    live_flush_interval: std::time::Duration,
    live_flush_bytes: usize,
    live_last_flush: Instant,
    #[cfg(not(target_family = "wasm"))]
    live_stream: Option<TcpStream>,
}

impl TelemetryMetrics {
    fn swf_display_name(input: &str) -> String {
        if let Ok(url) = Url::parse(input)
            && let Some(last) = url
                .path_segments()
                .and_then(|mut segments| segments.next_back())
            && !last.is_empty()
        {
            return last.to_string();
        }

        let without_query = input.split_once('?').map_or(input, |(prefix, _)| prefix);
        let without_query = without_query
            .split_once('#')
            .map_or(without_query, |(prefix, _)| prefix);

        without_query
            .rsplit(['/', '\\'])
            .find(|part| !part.is_empty())
            .unwrap_or(without_query)
            .to_string()
    }

    pub fn new(enabled: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch");
        let millis = now.as_millis() as f64;
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        let mut telemetry = Self {
            enabled,
            buffer: Vec::new(),
            string_table: Vec::new(),
            trait_table: Vec::new(),
            session_start_micros: now.as_micros(),
            last_event_micros: now.as_micros(),
            swf_name: "file:///ruffle_session.swf".to_string(),
            swf_rate_micros: 33333, // 30fps default
            swf_size_kb: 100,
            swf_width: 550,  // Default Flash stage width
            swf_height: 400, // Default Flash stage height
            swf_vm: 1,       // Default to AVM1
            swf_start_written: false,
            live_stream_offset: 0,
            // Keep telemetry feeling live while still batching writes.
            live_flush_interval: std::time::Duration::from_millis(100),
            live_flush_bytes: 4096,
            live_last_flush: Instant::now(),
            #[cfg(not(target_family = "wasm"))]
            live_stream: if enabled {
                Self::connect_live_stream()
            } else {
                None
            },
        };

        // Write session header metrics immediately
        // CRITICAL: .tlm.version MUST be the first metric
        telemetry.add_metric(".tlm.version", AmfValue::String("3,2".to_string()));
        telemetry.add_metric(".tlm.meta", AmfValue::Number(0.0));
        telemetry.add_metric(".tlm.date", AmfValue::Number(millis));

        // Player information
        telemetry.add_metric(".player.version", AmfValue::String("32,0,0,0".to_string()));
        telemetry.add_metric(".player.type", AmfValue::String("Standalone".to_string()));
        telemetry.add_metric(".player.debugger", AmfValue::Bool(false));
        telemetry.add_metric(".player.global.date", AmfValue::Number(millis));
        telemetry.add_metric(".player.instance", AmfValue::Integer(1));
        telemetry.add_metric(".player.scriptplayerversion", AmfValue::Integer(21));

        // Platform information
        let capabilities = format!("&M=Ruffle&COL=color&AR=1.0&OS={}&ARCH={}&L=en", os, arch);
        telemetry.add_metric(".platform.capabilities", AmfValue::String(capabilities));

        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get() as i32)
            .unwrap_or(1);
        telemetry.add_metric(".platform.cpucount", AmfValue::Integer(cpu_count));

        // Player user agent
        telemetry.add_metric(
            ".player.useragent",
            AmfValue::String(format!(
                "Ruffle/{} ({}; {})",
                env!("CARGO_PKG_VERSION"),
                os,
                arch
            )),
        );

        telemetry
    }

    /// Add memory metrics (can be called periodically to update memory stats).
    /// All values are in kilobytes.
    ///
    /// - `process_rss_kb`: OS-level resident set size (never decreases after GC, used as a floor)
    /// - `gc_live_kb`: live GC-managed bytes (decreases after collection)
    /// - `gc_arena_kb`: total GC arena capacity (may not decrease until arena shrinks)
    /// - `bitmap_kb`: CPU-side BitmapData + GPU-registered character bitmaps
    /// - `sound_kb`: compressed audio data held in the audio backend
    pub fn add_memory_metrics(
        &mut self,
        process_rss_kb: i32,
        gc_live_kb: i32,
        gc_arena_kb: i32,
        bitmap_kb: i32,
        sound_kb: i32,
    ) {
        let telemetry_kb = (self.buffer.len() / 1024) as i32;
        // Use the sum of tracked categories as "used", so GC frees are visible.
        // Fall back to RSS when the sum would exceed it (e.g. GPU memory not tracked).
        let tracked_kb = gc_live_kb + bitmap_kb + sound_kb + telemetry_kb;
        let total_kb = process_rss_kb.max(tracked_kb);
        self.add_metric(".mem.total", AmfValue::Integer(total_kb));
        self.add_metric(".mem.used", AmfValue::Integer(tracked_kb));
        // .mem.managed = arena capacity; .mem.managed.used = live GC bytes
        self.add_metric(".mem.managed", AmfValue::Integer(gc_arena_kb));
        self.add_metric(".mem.managed.used", AmfValue::Integer(gc_live_kb));
        self.add_metric(".mem.bitmap", AmfValue::Integer(bitmap_kb));
        self.add_metric(".mem.bitmap.display", AmfValue::Integer(bitmap_kb));
        // .mem.script is the live script heap — same as gc_live_kb
        self.add_metric(".mem.script", AmfValue::Integer(gc_live_kb));
        // Disabled: this metric causes Adobe Scout to fail to load real telemetry sessions.
        // self.add_metric(".mem.sound", AmfValue::Integer(sound_kb));
        self.add_metric(".mem.network.shared", AmfValue::Integer(0));
        self.add_metric(".mem.otherinstances", AmfValue::Integer(0));
        self.add_metric(".mem.telemetry.overhead", AmfValue::Integer(telemetry_kb));
    }

    /// Add a metric (any name/value pair) directly to the buffer
    /// Uses the .value trait class
    fn add_metric(&mut self, name: &str, value: AmfValue) {
        if !self.enabled {
            return;
        }
        // 0A = Object type marker
        self.buffer.push(0x0A);

        // Check if ".value" trait has been defined
        if let Some(trait_index) = self.trait_table.iter().position(|t| t == ".value") {
            // Reference existing trait
            let u29 = (trait_index << 2) | 1;
            self.write_u29(u29 as u32);
        } else {
            // Define new trait with class name ".value" and 2 sealed properties
            // 0x23 = 0b100011 = inline trait, not externalizable, not dynamic, 2 sealed properties
            self.buffer.push(0x23);

            // Class name: ".value"
            self.write_string_ref(".value");

            // Sealed property names
            self.write_string_ref("name");
            self.write_string_ref("value");

            // Add to trait table
            self.trait_table.push(".value".to_string());
        }

        // Property 1: name (the metric name)
        self.write_amf3_value(&AmfValue::String(name.to_string()));

        // Property 2: value (the metric value)
        self.write_amf3_value(&value);
        self.maybe_flush_live_stream();
    }

    /// Returns `Some(Instant::now())` when telemetry is enabled, otherwise `None`.
    pub fn now(&self) -> Option<Instant> {
        self.enabled.then(Instant::now)
    }

    /// Whether telemetry recording is enabled. Use this to skip work that's
    /// only needed to compute metric values (e.g. syscalls), as opposed to
    /// spans guarded by [`Self::now`].
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Record a span from a previously captured optional start time.
    pub fn add_span_from(&mut self, name: &str, start: Option<Instant>) {
        if let Some(start) = start {
            self.add_span(name, start.elapsed().as_micros() as i32);
        }
    }

    /// Start a timing span that will automatically record on drop (RAII pattern)
    /// Usage: let _span = telemetry.start_span(".rend.drawframe");
    /// Span will be recorded when _span goes out of scope
    pub fn start_span(&mut self, name: impl Into<String>) -> SpanGuard<'_> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch");

        SpanGuard {
            telemetry: self,
            name: name.into(),
            start_micros: now.as_micros(),
        }
    }

    /// Add a timing span metric (for performance profiling)
    /// - name: Metric name (e.g., ".rend.drawframe", ".as.doactions")
    /// - span_micros: Duration of the operation in microseconds
    ///
    /// Note: Delta is calculated automatically from the last event
    pub fn add_span(&mut self, name: &str, span_micros: i32) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();

        // Calculate delta from LAST EVENT, not session start
        let delta = (now_micros - self.last_event_micros) as i32;

        self.write_span(name, span_micros, delta);

        // Update last event time
        self.last_event_micros = now_micros;
    }

    /// Add a timing span metric using start/end timestamps
    /// Note: Delta is calculated automatically from the last event
    pub fn add_span_timed(&mut self, name: &str, start_micros: u128, end_micros: u128) {
        let span = (end_micros - start_micros) as i32;
        let delta = (end_micros - self.last_event_micros) as i32;

        self.write_span(name, span, delta);

        // Update last event time
        self.last_event_micros = end_micros;
    }

    /// Add a frame marker (value metric with frame number)
    /// This is what the Python analyzer looks for to count frames
    /// In Flash Player telemetry, .swf.frame is a value metric that's always 0
    /// Frames are counted by the number of .swf.frame markers, not their values
    /// Note: Delta is calculated automatically from the last event
    pub fn add_frame(&mut self, _frame_number: i32) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();

        // Frame number is ALWAYS 0 in Flash Player telemetry
        self.add_metric(".swf.frame", AmfValue::Integer(0));

        // Update last event time
        self.last_event_micros = now_micros;
    }

    /// Add a frame marker using a specific timestamp
    /// For testing or when you need precise control over event timing
    pub fn add_frame_timed(&mut self, _frame_number: i32, time_micros: u128) {
        // Frame number is ALWAYS 0 in Flash Player telemetry
        self.add_metric(".swf.frame", AmfValue::Integer(0));

        // Update last event time
        self.last_event_micros = time_micros;
    }

    /// Add a time marker (for frame boundaries like .enter, .exit)
    /// These use the .time trait class in Flash Player telemetry
    /// Note: Delta is calculated automatically from the last event
    pub fn add_time_marker(&mut self, name: &str) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();

        let delta = (now_micros - self.last_event_micros) as i32;
        self.write_time_marker(name, delta);

        // Update last event time
        self.last_event_micros = now_micros;
    }

    /// Add a time marker using a specific timestamp
    /// For testing or when you need precise control over event timing
    pub fn add_time_marker_timed(&mut self, name: &str, time_micros: u128) {
        let delta = (time_micros - self.last_event_micros) as i32;
        self.write_time_marker(name, delta);

        // Update last event time
        self.last_event_micros = time_micros;
    }

    /// Mark the start of frame execution (.enter marker in Flash Player)
    /// This is written BEFORE the .swf.frame marker and frame's span metrics
    pub fn add_frame_enter(&mut self) {
        self.add_time_marker(".enter");
    }

    /// Mark the end of frame execution (.exit marker in Flash Player)
    /// This is written AFTER all the frame's span metrics
    pub fn add_frame_exit(&mut self) {
        self.add_time_marker(".exit");
    }

    /// Set the SWF name metric (call after loading a SWF)
    pub fn set_swf_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        self.swf_name = Self::swf_display_name(&name);
    }

    /// Set the SWF frame rate in fps (call after loading a SWF)
    /// Frame rate is stored as microseconds per frame
    pub fn set_swf_rate(&mut self, fps: f64) {
        self.swf_rate_micros = (1_000_000.0 / fps) as i32;
    }

    /// Set the SWF file size in kilobytes (call after loading a SWF)
    pub fn set_swf_size(&mut self, size_kb: i32) {
        self.swf_size_kb = size_kb;
    }

    /// Set the SWF dimensions in pixels (call after loading a SWF)
    pub fn set_swf_dimensions(&mut self, width: i32, height: i32) {
        self.swf_width = width;
        self.swf_height = height;
    }

    /// Set the ActionScript VM version (1 = AVM1, 2 = AVM2)
    pub fn set_swf_vm(&mut self, vm: i32) {
        self.swf_vm = vm;
    }

    /// Set the SWF start time and write SWF info section
    /// This writes .swf.start marker followed by .swf.name, .swf.rate, .swf.size
    /// Call this when SWF begins execution
    pub fn set_swf_start(&mut self, delta_micros: i32) {
        if self.swf_start_written {
            return; // Already written
        }

        // Write .swf.start marker using .time trait
        self.write_time_marker(".swf.start", delta_micros);

        // Write SWF info metrics
        self.add_metric(".swf.name", AmfValue::String(self.swf_name.clone()));
        self.add_metric(".swf.rate", AmfValue::Integer(self.swf_rate_micros));
        self.add_metric(".swf.size", AmfValue::Integer(self.swf_size_kb));
        self.add_metric(".swf.width", AmfValue::Integer(self.swf_width));
        self.add_metric(".swf.height", AmfValue::Integer(self.swf_height));
        self.add_metric(".swf.vm", AmfValue::Integer(self.swf_vm));

        self.swf_start_written = true;
    }

    /// Record a network load movie event (.network.loadmovie with URL)
    pub fn add_network_loadmovie(&mut self, url: &str) {
        self.add_metric(".network.loadmovie", AmfValue::String(url.to_string()));
    }

    /// Record network data received (.network.swf.received with size in KB)
    pub fn add_network_received(&mut self, size_kb: i32) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();
        let delta = (now_micros - self.last_event_micros) as i32;
        self.write_span_value_int(".network.swf.received", 1, delta, size_kb);
        self.last_event_micros = now_micros;
    }

    /// Record a render update region (.rend.update with a .region value)
    /// span_micros: duration of the render update in microseconds
    /// xmin/xmax/ymin/ymax: pixel coordinates of the dirty rectangle
    pub fn add_rend_update(
        &mut self,
        span_micros: i32,
        xmin: i32,
        xmax: i32,
        ymin: i32,
        ymax: i32,
    ) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();

        let delta = (now_micros - self.last_event_micros) as i32;
        self.write_span_value_region(".rend.update", span_micros, delta, xmin, xmax, ymin, ymax);
        self.last_event_micros = now_micros;
    }

    /// Record a render update region using explicit timestamps
    pub fn add_rend_update_timed(
        &mut self,
        start_micros: u128,
        end_micros: u128,
        xmin: i32,
        xmax: i32,
        ymin: i32,
        ymax: i32,
    ) {
        let span = (end_micros - start_micros) as i32;
        let delta = (end_micros - self.last_event_micros) as i32;
        self.write_span_value_region(".rend.update", span, delta, xmin, xmax, ymin, ymax);
        self.last_event_micros = end_micros;
    }

    /// Helper: Write a span metric directly to buffer
    fn write_span(&mut self, name: &str, span: i32, delta: i32) {
        // 0A = Object type marker
        self.buffer.push(0x0A);

        // Check if ".span" trait has been defined
        if let Some(trait_index) = self.trait_table.iter().position(|t| t == ".span") {
            // Reference existing trait
            let u29 = (trait_index << 2) | 1;
            self.write_u29(u29 as u32);
        } else {
            // Define new trait with class name ".span" and 3 sealed properties
            // 0x33 = 0b110011 = inline trait, not externalizable, not dynamic, 3 sealed properties
            self.buffer.push(0x33);

            // Class name: ".span"
            self.write_string_ref(".span");

            // Sealed property names
            self.write_string_ref("name");
            self.write_string_ref("span");
            self.write_string_ref("delta");

            // Add to trait table
            self.trait_table.push(".span".to_string());
        }

        // Property values
        self.write_amf3_value(&AmfValue::String(name.to_string()));
        self.write_amf3_value(&AmfValue::Integer(span));
        self.write_amf3_value(&AmfValue::Integer(delta));
        self.maybe_flush_live_stream();
    }

    /// Helper: Write a time marker directly to buffer
    fn write_time_marker(&mut self, name: &str, delta: i32) {
        // 0A = Object type marker
        self.buffer.push(0x0A);

        // Check if ".time" trait has been defined
        if let Some(trait_index) = self.trait_table.iter().position(|t| t == ".time") {
            // Reference existing trait
            let u29 = (trait_index << 2) | 1;
            self.write_u29(u29 as u32);
        } else {
            // Define new trait with class name ".time" and 2 sealed properties
            // 0x23 = 0b100011 = inline trait, not externalizable, not dynamic, 2 sealed properties
            self.buffer.push(0x23);

            // Class name: ".time"
            self.write_string_ref(".time");

            // Sealed property names
            self.write_string_ref("name");
            self.write_string_ref("delta");

            // Add to trait table
            self.trait_table.push(".time".to_string());
        }

        // Property values
        self.write_amf3_value(&AmfValue::String(name.to_string()));
        self.write_amf3_value(&AmfValue::Integer(delta));
        self.maybe_flush_live_stream();
    }

    /// Record a key down event (.player.key.down with key code value)
    pub fn add_key_down(&mut self, span_micros: i32, key_code: i32) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();
        let delta = (now_micros - self.last_event_micros) as i32;
        self.write_span_value_int(".player.key.down", span_micros, delta, key_code);
        self.last_event_micros = now_micros;
    }

    /// Record a key press event (.player.key.press with key code value)
    pub fn add_key_press(&mut self, span_micros: i32, key_code: i32) {
        let now_micros = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_micros();
        let delta = (now_micros - self.last_event_micros) as i32;
        self.write_span_value_int(".player.key.press", span_micros, delta, key_code);
        self.last_event_micros = now_micros;
    }

    /// Helper: Write .spanValue header (name, span, delta) — caller must write the value property
    fn write_span_value_header(&mut self, name: &str, span: i32, delta: i32) {
        self.buffer.push(0x0A);

        if let Some(trait_index) = self.trait_table.iter().position(|t| t == ".spanValue") {
            let u29 = (trait_index << 2) | 1;
            self.write_u29(u29 as u32);
        } else {
            // 0x43 = 4 sealed properties, inline trait, not externalizable, not dynamic
            self.buffer.push(0x43);
            self.write_string_ref(".spanValue");
            self.write_string_ref("name");
            self.write_string_ref("span");
            self.write_string_ref("delta");
            self.write_string_ref("value");
            self.trait_table.push(".spanValue".to_string());
        }

        self.write_amf3_value(&AmfValue::String(name.to_string()));
        self.write_amf3_value(&AmfValue::Integer(span));
        self.write_amf3_value(&AmfValue::Integer(delta));
    }

    /// Helper: Write a .spanValue with a .region object as the value
    #[allow(clippy::too_many_arguments)]
    fn write_span_value_region(
        &mut self,
        name: &str,
        span: i32,
        delta: i32,
        xmin: i32,
        xmax: i32,
        ymin: i32,
        ymax: i32,
    ) {
        self.write_span_value_header(name, span, delta);
        self.write_region(xmin, xmax, ymin, ymax);
        self.maybe_flush_live_stream();
    }

    /// Helper: Write a .spanValue with an integer as the value
    fn write_span_value_int(&mut self, name: &str, span: i32, delta: i32, value: i32) {
        self.write_span_value_header(name, span, delta);
        self.write_amf3_value(&AmfValue::Integer(value));
        self.maybe_flush_live_stream();
    }

    /// Helper: Write a .region object (xmin, xmax, ymin, ymax, name, symbolname, modified)
    fn write_region(&mut self, xmin: i32, xmax: i32, ymin: i32, ymax: i32) {
        self.buffer.push(0x0A);

        if let Some(trait_index) = self.trait_table.iter().position(|t| t == ".region") {
            let u29 = (trait_index << 2) | 1;
            self.write_u29(u29 as u32);
        } else {
            // 0x73 = 7 sealed properties, inline trait, not externalizable, not dynamic
            self.buffer.push(0x73);
            self.write_string_ref(".region");
            self.write_string_ref("xmin");
            self.write_string_ref("xmax");
            self.write_string_ref("ymin");
            self.write_string_ref("ymax");
            self.write_string_ref("name");
            self.write_string_ref("symbolname");
            self.write_string_ref("modified");
            self.trait_table.push(".region".to_string());
        }

        self.write_amf3_value(&AmfValue::Integer(xmin));
        self.write_amf3_value(&AmfValue::Integer(xmax));
        self.write_amf3_value(&AmfValue::Integer(ymin));
        self.write_amf3_value(&AmfValue::Integer(ymax));
        self.buffer.push(0x01); // name = Null
        self.buffer.push(0x01); // symbolname = Null
        self.buffer.push(0x02); // modified = False
    }

    pub fn write_flm(&self, path: &str) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        let mut file = File::create(path)?;
        file.write_all(&self.buffer)?;
        file.flush()?;
        Ok(())
    }

    #[cfg(not(target_family = "wasm"))]
    fn connect_live_stream() -> Option<TcpStream> {
        match TcpStream::connect("127.0.0.1:7934") {
            Ok(stream) => {
                let _ = stream.set_nodelay(true);
                Some(stream)
            }
            Err(error) => {
                tracing::debug!("Telemetry live stream unavailable on 127.0.0.1:7934: {error}");
                None
            }
        }
    }

    fn maybe_flush_live_stream(&mut self) {
        if !self.enabled {
            return;
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let elapsed = self.live_last_flush.elapsed();
            let interval = self.live_flush_interval;
            let flush_bytes = self.live_flush_bytes;
            let Some(stream) = self.live_stream.as_mut() else {
                return;
            };

            let pending = self.buffer.len().saturating_sub(self.live_stream_offset);
            if !Self::should_flush_live_stream(pending, elapsed, interval, flush_bytes) {
                return;
            }

            if stream
                .write_all(&self.buffer[self.live_stream_offset..])
                .is_ok()
            {
                self.live_stream_offset = self.buffer.len();
                self.live_last_flush = Instant::now();
            } else {
                tracing::warn!(
                    "Telemetry live stream disconnected; continuing local buffering only"
                );
                self.live_stream = None;
            }
        }
    }

    fn should_flush_live_stream(
        pending: usize,
        elapsed: std::time::Duration,
        interval: std::time::Duration,
        flush_bytes: usize,
    ) -> bool {
        pending > 0 && (elapsed >= interval || pending >= flush_bytes)
    }

    fn write_string_ref(&mut self, s: &str) {
        // Check if string is already in the reference table
        if let Some(index) = self.string_table.iter().position(|x| x == s) {
            // Write reference to existing string (index << 1, with bit 0 = 0 for reference)
            let u29 = (index << 1) as u32;
            self.write_u29(u29);
        } else {
            // Write inline string and add to reference table
            let len = s.len();
            let u29 = (len << 1) | 1; // Set reference bit to indicate inline string
            self.write_u29(u29 as u32);
            self.buffer.extend_from_slice(s.as_bytes());
            self.string_table.push(s.to_string());
        }
    }

    fn write_amf3_value(&mut self, value: &AmfValue) {
        match value {
            AmfValue::String(s) => {
                self.buffer.push(0x06); // String type marker
                self.write_string_ref(s);
            }
            AmfValue::Number(n) => {
                self.buffer.push(0x05);
                self.buffer.extend_from_slice(&n.to_be_bytes());
            }
            AmfValue::Integer(i) => {
                self.buffer.push(0x04);
                self.write_u29(*i as u32);
            }
            AmfValue::Bool(b) => {
                if *b {
                    self.buffer.push(0x03); // true
                } else {
                    self.buffer.push(0x02); // false
                }
            }
            _ => {}
        }
    }

    fn write_u29(&mut self, value: u32) {
        // AMF3 variable length unsigned 29-bit integer encoding
        if value < 0x80 {
            self.buffer.push(value as u8);
        } else if value < 0x4000 {
            self.buffer.push(((value >> 7) | 0x80) as u8);
            self.buffer.push((value & 0x7F) as u8);
        } else if value < 0x200000 {
            self.buffer.push(((value >> 14) | 0x80) as u8);
            self.buffer.push((((value >> 7) & 0x7F) | 0x80) as u8);
            self.buffer.push((value & 0x7F) as u8);
        } else {
            self.buffer.push(((value >> 22) | 0x80) as u8);
            self.buffer.push((((value >> 15) & 0x7F) | 0x80) as u8);
            self.buffer.push((((value >> 8) & 0x7F) | 0x80) as u8);
            self.buffer.push((value & 0xFF) as u8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_flm() {
        let telemetry = TelemetryMetrics::new(true);
        let path = std::env::temp_dir().join("ruffle_test_output.flm");
        let path = path.to_str().expect("Temp path should be valid UTF-8");
        telemetry
            .write_flm(path)
            .expect("Failed to write .flm file");

        // Read back the file and check contents
        let _data = fs::read(path).expect("Failed to read .flm file");
        // Print hex dump of first 256 bytes for comparison with real .flm files
        println!(
            "First {} bytes of generated .flm file:",
            _data.len().min(256)
        );
        for (i, chunk) in _data.chunks(16).take(16).enumerate() {
            print!("{:08x}: ", i * 16);
            for byte in chunk {
                print!("{:02x} ", byte);
            }
            println!();
        }
        // TODO: Add AMF3 deserialization check here if available
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_set_swf_name() {
        let mut telemetry = TelemetryMetrics::new(true);
        telemetry.set_swf_name("test.swf");

        // Just verify it doesn't crash - actual value is written when set_swf_start is called
        assert_eq!(telemetry.swf_name, "test.swf");
    }

    #[test]
    fn test_set_swf_name_from_url() {
        let mut telemetry = TelemetryMetrics::new(true);
        telemetry.set_swf_name("file:///Users/test/Downloads/test.swf?foo=bar#section");

        assert_eq!(telemetry.swf_name, "test.swf");
    }

    #[test]
    fn test_set_swf_rate() {
        let mut telemetry = TelemetryMetrics::new(true);
        telemetry.set_swf_rate(60.0); // 60fps

        // 60fps = 1_000_000 / 60 = 16666 microseconds per frame
        assert_eq!(telemetry.swf_rate_micros, 16666);
    }

    #[test]
    fn test_set_swf_start() {
        let mut telemetry = TelemetryMetrics::new(true);
        telemetry.set_swf_name("test.swf");
        telemetry.set_swf_rate(30.0);
        telemetry.set_swf_size(1024);
        telemetry.set_swf_start(12345);

        // Verify .swf.start marker and info were written to buffer
        assert!(
            telemetry.swf_start_written,
            "SWF start should be marked as written"
        );
        assert!(
            telemetry.buffer.len() > 100,
            "Buffer should contain session + swf data"
        );
    }

    #[test]
    fn test_add_span() {
        let mut telemetry = TelemetryMetrics::new(true);

        // Set SWF info for more realistic session
        telemetry.set_swf_name("test_animation.swf");
        telemetry.set_swf_rate(30.0); // 30 FPS = 33.33ms per frame
        telemetry.set_swf_size(2048); // 2MB
        telemetry.set_swf_start(118); // .swf.start marker at 118us (like log7.flm)

        // Simulate 10 frames of activity with realistic timing
        // 30 FPS = 33,333 microseconds per frame
        // We'll use add_span_timed() to control exact timing without thread sleeps

        let start = telemetry.session_start_micros;
        let mut current_time = start + 118; // Start after .swf.start delta

        for frame_num in 0..10 {
            // Each frame takes ~33ms at 30 FPS
            let frame_start = current_time;

            // 1. Frame enter marker (2μs after previous frame ended)
            current_time += 2;
            telemetry.add_time_marker_timed(".enter", current_time);

            // 2. Frame marker (1μs after enter)
            current_time += 1;
            telemetry.add_frame_timed(frame_num, current_time);

            // 3. Frame work spans (must not overlap!)
            // Player overhead (.tlm.doplay) - typically 5-10ms
            let doplay_start = current_time + 1;
            let doplay_duration: u128 = 8000; // 8ms
            telemetry.add_span_timed(".tlm.doplay", doplay_start, doplay_start + doplay_duration);
            current_time = doplay_start + doplay_duration;

            // Rendering - typically 10-15ms, varies slightly per frame
            let render_start = current_time + 1;
            let render_duration: u128 = 12000 + (frame_num as u128 * 100); // 12-13ms
            telemetry.add_span_timed(
                ".rend.drawframe",
                render_start,
                render_start + render_duration,
            );
            current_time = render_start + render_duration;

            // ActionScript execution - typically 5-10ms
            let as_start = current_time + 1;
            let as_duration: u128 = 7000 + ((frame_num % 3) as u128 * 500); // 7-8.5ms
            telemetry.add_span_timed(".as.doactions", as_start, as_start + as_duration);
            current_time = as_start + as_duration;

            // Occasional GC (every 4 frames)
            if frame_num % 4 == 0 {
                let gc_start = current_time + 1;
                let gc_duration: u128 = 3000; // 3ms GC
                telemetry.add_span_timed(".gc.mark", gc_start, gc_start + gc_duration);
                current_time = gc_start + gc_duration;
            }

            // 4. Frame exit marker (1μs after last operation)
            current_time += 1;
            telemetry.add_time_marker_timed(".exit", current_time);

            // Advance to next frame (pad remaining time to hit 33.33ms per frame)
            let frame_duration = current_time - frame_start;
            let target_frame_time: u128 = 33333; // 33.33ms at 30 FPS
            if frame_duration < target_frame_time {
                current_time += target_frame_time - frame_duration;
            }
        }

        // Write to file and check that expected trait classes are encoded
        let path = std::env::temp_dir().join("ruffle_test_spans.flm");
        let path = path.to_str().expect("Temp path should be valid UTF-8");
        telemetry
            .write_flm(path)
            .expect("Failed to write .flm with spans");

        let data = fs::read(path).expect("Failed to read .flm file");

        // Verify file contains expected trait definitions
        let value_bytes = b".value";
        let span_bytes = b".span";
        let time_bytes = b".time";
        assert!(
            data.windows(value_bytes.len()).any(|w| w == value_bytes),
            "File should contain .value trait class"
        );
        assert!(
            data.windows(span_bytes.len()).any(|w| w == span_bytes),
            "File should contain .span trait class"
        );
        assert!(
            data.windows(time_bytes.len()).any(|w| w == time_bytes),
            "File should contain .time trait class"
        );

        println!("\nGenerated {path}:");
        println!("  File size: {} bytes", data.len());
        println!("\nTo analyze:");
        println!("  python3 telemetry.py {path}");
        println!("  python3 telemetry.py -f {path}  # Frame-by-frame");
        println!("\nOr open {path} in Adobe Scout");

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_rend_update() {
        let mut telemetry = TelemetryMetrics::new(true);
        telemetry.set_swf_name("test.swf");
        telemetry.set_swf_rate(30.0);
        telemetry.set_swf_size(1024);
        telemetry.set_swf_dimensions(550, 400);
        telemetry.set_swf_vm(2);
        telemetry.set_swf_start(0);

        let start = telemetry.session_start_micros;
        let mut t = start + 100;

        // Frame 0: .enter, .swf.frame, .as.doactions, .rend.update, .rend.drawframe, .exit
        t += 2;
        telemetry.add_time_marker_timed(".enter", t);
        t += 1;
        telemetry.add_frame_timed(0, t);

        let as_start = t + 1;
        telemetry.add_span_timed(".as.doactions", as_start, as_start + 5000);
        t = as_start + 5000;

        let rend_start = t + 1;
        let rend_end = rend_start + 8000;
        telemetry.add_rend_update_timed(rend_start, rend_end, 0, 550, 0, 400);
        telemetry.add_span_timed(".rend.drawframe", rend_start, rend_end);
        t = rend_end;

        t += 1;
        telemetry.add_time_marker_timed(".exit", t);

        let path = std::env::temp_dir().join("ruffle_test_rend_update.flm");
        let path = path.to_str().expect("Temp path should be valid UTF-8");
        telemetry.write_flm(path).expect("Failed to write .flm");

        let data = fs::read(path).expect("Failed to read .flm file");
        println!("Generated {path} ({} bytes)", data.len());
        println!("Run: python3 telemetry.py {path}");

        let sv_bytes = b".spanValue";
        let region_bytes = b".region";
        assert!(
            data.windows(sv_bytes.len()).any(|w| w == sv_bytes),
            "File should contain .spanValue trait class"
        );
        assert!(
            data.windows(region_bytes.len()).any(|w| w == region_bytes),
            "File should contain .region trait class"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_add_span_timed() {
        let mut telemetry = TelemetryMetrics::new(true);

        let start = telemetry.session_start_micros;
        let frame_start = start + 1000;
        let frame_end = frame_start + 16666;

        telemetry.add_span_timed(".swf.frame", frame_start, frame_end);

        // Just verify it doesn't crash and produces output
        let path = std::env::temp_dir().join("ruffle_test_span_timed.flm");
        let path = path.to_str().expect("Temp path should be valid UTF-8");
        telemetry.write_flm(path).expect("Failed to write .flm");
        let data = fs::read(path).expect("Failed to read .flm file");
        assert!(data.len() > 100, "File should have reasonable size");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_live_flush_policy() {
        let telemetry = TelemetryMetrics::new(false);
        let interval = telemetry.live_flush_interval;
        let flush_bytes = telemetry.live_flush_bytes;

        assert!(!TelemetryMetrics::should_flush_live_stream(
            0,
            std::time::Duration::from_millis(500),
            interval,
            flush_bytes,
        ));
        assert!(!TelemetryMetrics::should_flush_live_stream(
            256,
            std::time::Duration::from_millis(20),
            interval,
            flush_bytes,
        ));
        assert!(TelemetryMetrics::should_flush_live_stream(
            256,
            std::time::Duration::from_millis(100),
            interval,
            flush_bytes,
        ));
        assert!(TelemetryMetrics::should_flush_live_stream(
            4096,
            std::time::Duration::from_millis(1),
            interval,
            flush_bytes,
        ));
    }
}
