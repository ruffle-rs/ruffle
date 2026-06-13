#![no_std]

//! # is_wine
//!
//! A library to easily check if the current app is running under wine.

mod core {
    use core::fmt;

    /// An error araised by is_wine.
    #[derive(Debug, Clone)]
    pub struct IsWineError;

    impl fmt::Display for IsWineError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "failed to check wine runtime")
        }
    }

    /// The is_wine type-safe result type.
    pub type IsWineResult<T> = Result<T, IsWineError>;
}

#[cfg(all(windows, not(doc)))]
mod platform {
    use windows::{
        core::{s, PCSTR},
        Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
    };

    use crate::{core::IsWineResult, IsWineError};

    const NTDLL_MODULE: PCSTR = s!("ntdll.dll");
    const WINE_DETECTION_PROC: PCSTR = s!("wine_get_version");

    pub fn is_wine() -> bool {
        try_is_wine().unwrap()
    }

    pub fn try_is_wine() -> IsWineResult<bool> {
        let module_handle = unsafe { GetModuleHandleA(NTDLL_MODULE) }.map_err(|_| IsWineError)?;

        // If ntdll module could not be found we return an error.
        // NOTE: This should never happen normally but if it does we have a sane default.
        if module_handle.is_invalid() {
            return Err(IsWineError);
        }

        let address = unsafe { GetProcAddress(module_handle, WINE_DETECTION_PROC) };
        let detected_wine_symbol = address.is_some();
        Ok(detected_wine_symbol)
    }

    pub fn is_wine_lax() -> bool {
        try_is_wine().unwrap_or(false)
    }
}

// Applications not running under windows cannot be running under wine
#[cfg(all(not(windows), not(doc)))]
mod platform {
    use crate::core::IsWineResult;
    pub fn is_wine() -> bool {
        false
    }

    pub fn try_is_wine() -> IsWineResult<bool> {
        Ok(false)
    }

    pub fn is_wine_lax() -> bool {
        false
    }
}

// Documentation for platform dependent symbols
#[cfg(doc)]
mod platform {
    use crate::core::IsWineResult;
    /// Check if app is running under wine. Panics on failure.
    pub fn is_wine() -> bool {
        unimplemented!()
    }

    /// Check if app is running under wine. Returns an error on failure.
    pub fn try_is_wine() -> IsWineResult<bool> {
        unimplemented!()
    }

    /// Check if app is running under wine. Returns false on failure.
    pub fn is_wine_lax() -> bool {
        unimplemented!()
    }
}

pub use core::*;
pub use platform::*;
