#[cfg(windows)]
fn main() {
    embed_resource::compile("assets/ruffle_desktop.rc")
}

#[cfg(not(windows))]
fn main() {}