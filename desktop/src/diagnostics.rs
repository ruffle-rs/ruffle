#[cfg(windows)]
mod imp {
    use std::mem::size_of;
    use std::time::Instant;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    #[derive(Clone, Copy, Debug)]
    pub struct ProcessMetrics {
        pub working_set_bytes: u64,
        pub peak_working_set_bytes: u64,
        pub private_bytes: u64,
        pub cpu_percent: Option<f64>,
    }

    #[derive(Debug)]
    pub struct ProcessDiagnosticsSampler {
        logical_cpus: f64,
        last_sample: Option<Instant>,
        last_process_time_100ns: Option<u64>,
    }

    impl ProcessDiagnosticsSampler {
        pub fn new() -> Self {
            Self {
                logical_cpus: std::thread::available_parallelism()
                    .map(|count| count.get() as f64)
                    .unwrap_or(1.0),
                last_sample: Some(Instant::now()),
                last_process_time_100ns: process_time_100ns(),
            }
        }

        pub fn sample(&mut self) -> Option<ProcessMetrics> {
            let now = Instant::now();
            let process_time_100ns = process_time_100ns();
            let cpu_percent = match (
                self.last_sample,
                self.last_process_time_100ns,
                process_time_100ns,
            ) {
                (Some(last_sample), Some(last_process_time), Some(process_time)) => {
                    let elapsed = now.duration_since(last_sample).as_secs_f64();
                    if elapsed > 0.0 && process_time >= last_process_time {
                        let process_elapsed =
                            (process_time - last_process_time) as f64 / 10_000_000.0;
                        Some((process_elapsed / elapsed / self.logical_cpus) * 100.0)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            self.last_sample = Some(now);
            self.last_process_time_100ns = process_time_100ns;

            process_memory().map(|memory| ProcessMetrics {
                working_set_bytes: memory.working_set_bytes,
                peak_working_set_bytes: memory.peak_working_set_bytes,
                private_bytes: memory.private_bytes,
                cpu_percent,
            })
        }
    }

    struct ProcessMemory {
        working_set_bytes: u64,
        peak_working_set_bytes: u64,
        private_bytes: u64,
    }

    fn process_memory() -> Option<ProcessMemory> {
        let mut counters = PROCESS_MEMORY_COUNTERS_EX {
            cb: size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            PageFaultCount: 0,
            PeakWorkingSetSize: 0,
            WorkingSetSize: 0,
            QuotaPeakPagedPoolUsage: 0,
            QuotaPagedPoolUsage: 0,
            QuotaPeakNonPagedPoolUsage: 0,
            QuotaNonPagedPoolUsage: 0,
            PagefileUsage: 0,
            PeakPagefileUsage: 0,
            PrivateUsage: 0,
        };

        let ok = unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
                size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32,
            )
        };

        if ok == 0 {
            return None;
        }

        Some(ProcessMemory {
            working_set_bytes: counters.WorkingSetSize as u64,
            peak_working_set_bytes: counters.PeakWorkingSetSize as u64,
            private_bytes: counters.PrivateUsage as u64,
        })
    }

    fn process_time_100ns() -> Option<u64> {
        let mut creation_time = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut exit_time = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut kernel_time = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut user_time = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };

        let ok = unsafe {
            GetProcessTimes(
                GetCurrentProcess(),
                &mut creation_time,
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            )
        };

        if ok == 0 {
            return None;
        }

        Some(filetime_to_u64(kernel_time).saturating_add(filetime_to_u64(user_time)))
    }

    fn filetime_to_u64(filetime: FILETIME) -> u64 {
        (u64::from(filetime.dwHighDateTime) << 32) | u64::from(filetime.dwLowDateTime)
    }
}

#[cfg(not(windows))]
mod imp {
    #[derive(Clone, Copy, Debug)]
    pub struct ProcessMetrics {
        pub working_set_bytes: u64,
        pub peak_working_set_bytes: u64,
        pub private_bytes: u64,
        pub cpu_percent: Option<f64>,
    }

    #[derive(Debug)]
    pub struct ProcessDiagnosticsSampler;

    impl ProcessDiagnosticsSampler {
        pub fn new() -> Self {
            Self
        }

        pub fn sample(&mut self) -> Option<ProcessMetrics> {
            None
        }
    }
}

pub use imp::{ProcessDiagnosticsSampler, ProcessMetrics};
