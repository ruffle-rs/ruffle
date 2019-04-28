use fluster_core::Player;

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn main() {
    use std::path::PathBuf;
    use structopt::StructOpt;

    #[derive(StructOpt, Debug)]
    #[structopt(name = "basic")]
    struct Opt {
        #[structopt(name = "FILE", parse(from_os_str))]
        input_path: PathBuf,
    }

    let opt = Opt::from_args();

    let swf_data = std::fs::read(opt.input_path).unwrap();
    let mut player = Player::new(swf_data).unwrap();
    player.play();
}
