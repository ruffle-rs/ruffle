use windows_sys::Win32::Storage::FileSystem::{FILE_TYPE_DISK, FILE_TYPE_PIPE, GetFileType};
use windows_sys::Win32::System::Console::{
    ATTACH_PARENT_PROCESS, AttachConsole, FreeConsole, GetStdHandle, STD_OUTPUT_HANDLE,
};

/// RAII guard for the attached parent console.
/// Frees the console on drop if it was successfully attached.
pub(super) struct Console {
    attached: bool,
}

impl Console {
    // When linked with the windows subsystem windows won't automatically attach
    // to the console of the parent process, so we do it explicitly. This fails
    // silently if the parent has no console.
    //
    // However, if stdout/stderr are already redirected (e.g., `ruffle.exe > file.txt`),
    // we should NOT attach to the console as that would bypass the redirection.
    // See: https://github.com/ruffle-rs/ruffle/issues/9145
    pub(super) fn attach() -> Self {
        // Check if stdout is already redirected to a file or pipe
        // SAFETY: STD_OUTPUT_HANDLE is a valid standard device constant.
        let stdout_handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };

        // GetStdHandle can return NULL on failure
        let attached = if stdout_handle.is_null() {
            false
        } else {
            // SAFETY: GetFileType accepts any handle value including INVALID_HANDLE_VALUE,
            // returning FILE_TYPE_UNKNOWN in that case.
            let file_type = unsafe { GetFileType(stdout_handle) };

            match file_type {
                // If output is redirected to a file or pipe, don't attach to console
                // as that would bypass the redirection
                FILE_TYPE_DISK | FILE_TYPE_PIPE => false,
                // Otherwise, attach to parent console for interactive use
                // SAFETY: ATTACH_PARENT_PROCESS is a valid constant for AttachConsole.
                // This call fails silently if the parent has no console.
                _ => (unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }) != 0,
            }
        };

        Self { attached }
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        if self.attached {
            // Without explicitly detaching, cmd won't redraw its prompt.
            // SAFETY: We only call FreeConsole if it was previously successfully attached.
            unsafe { FreeConsole() };
        }
    }
}
