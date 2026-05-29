cfg_select! {
    target_os = "linux" => {
        mod linux;
        pub use linux::*;
    }
    _ => {
        mod unsupported;
        pub use unsupported::*;
    }
}
