use crate::environment::RenderInterface;
use crate::options::image_comparison::ImageComparison;
use crate::util::{read_bytes, write_image};
use anyhow::anyhow;
use image::ImageFormat;
use ruffle_core::Player;
use std::sync::{Arc, Mutex};
use vfs::VfsPath;

pub fn capture_and_compare_image(
    base_path: &VfsPath,
    player: &Arc<Mutex<Player>>,
    name: &String,
    image_comparison: ImageComparison,
    known_failure: bool,
    render_interface: Option<&dyn RenderInterface>,
) -> anyhow::Result<()> {
    use anyhow::Context;

    if let Some(render_interface) = render_interface {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();

        let actual_image = render_interface.capture(player_lock.renderer_mut());

        let expected_image_path = base_path.join(format!("{name}.expected.png"))?;
        if expected_image_path.is_file()? {
            let expected_image = image::load_from_memory(&read_bytes(&expected_image_path)?)
                .context("Failed to open expected image")?
                .into_rgba8();

            image_comparison.test(
                name,
                actual_image,
                expected_image,
                base_path,
                render_interface.name(),
                known_failure,
            )?;
        } else if known_failure {
            return Err(anyhow!(
                "No image to compare to, pretending this failed since we don't know if it worked."
            ));
        } else {
            // If we're expecting this to be wrong, don't save a likely wrong image
            write_image(&expected_image_path, &actual_image, ImageFormat::Png)?;
        }
    } else if known_failure {
        // It's possible that the trace output matched but the image might not.
        // If we aren't checking the image, pretend the match failed (which makes it actually pass, since it's expecting failure).
        return Err(anyhow!(
            "Not checking images, pretending this failed since we don't know if it worked."
        ));
    }

    Ok(())
}
