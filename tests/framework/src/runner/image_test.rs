use crate::environment::RenderInterface;
use crate::options::image_comparison::ImageComparison;
use crate::util::{read_bytes, write_bytes};
use anyhow::{Context as _, anyhow};
use image::{EncodableLayout, ImageBuffer, ImageFormat, Pixel, PixelWithColorType};
use ruffle_core::Player;
use std::borrow::Cow;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use vfs::VfsPath;

pub fn capture_and_compare_image(
    base_path: &VfsPath,
    player: &Arc<Mutex<Player>>,
    name: &str,
    image_comparison: ImageComparison,
    render_interface: Option<&dyn RenderInterface>,
) -> anyhow::Result<()> {
    let Some(render_interface) = render_interface else {
        return Ok(());
    };

    let actual_image = {
        let mut player_lock = player.lock().unwrap();
        player_lock.render();
        render_interface.capture(player_lock.renderer_mut())
    };

    let expected_image = {
        let path = base_path.join(format!("{name}.expected.png"))?;
        if path.exists()? {
            image::load_from_memory(&read_bytes(&path)?)
                .context("Failed to open expected image")?
                .into_rgba8()
        } else if image_comparison.known_failure {
            // If we're expecting this to be wrong, don't save a likely wrong image
            return Err(anyhow!("Image '{name}': No image to compare to!"));
        } else {
            write_image(&path, &actual_image)?;
            return Err(anyhow!(
                "Image '{name}': No image to compare to! Saved actual image as expected."
            ));
        }
    };

    let ruffle_expected_path = base_path.join(format!("{name}.ruffle.png"))?;

    let diff = test(&image_comparison, name, &actual_image, expected_image)?;
    let (failure, failure_name) = match (diff, image_comparison.known_failure) {
        (None, false) => {
            return if ruffle_expected_path.exists()? {
                Err(anyhow!(
                    "Unexpected `{}` file for passing check, please remove it!",
                    ruffle_expected_path.as_str(),
                ))
            } else {
                Ok(())
            };
        }
        (None, true) => {
            return Err(anyhow!(
                "Image '{name}': Check was known to be failing, but now passes successfully. \
                Please update the test, and remove `known_failure = true` and `{name}.ruffle.png`!",
            ));
        }
        (Some(diff), false) => (diff, Cow::Borrowed(name)),
        (Some(_), true) => {
            let ruffle_name = format!("{name}.ruffle");
            let image = if ruffle_expected_path.exists()? {
                image::load_from_memory(&read_bytes(&ruffle_expected_path)?)
                    .context("Failed to open Ruffle-expected image")?
                    .into_rgba8()
            } else {
                write_image(&ruffle_expected_path, &actual_image)?;
                return Err(anyhow!(
                    "Image '{ruffle_name}': No image to compare to! Saved actual image as Ruffle-expected."
                ));
            };

            if let Some(diff) = test(&image_comparison, &ruffle_name, &actual_image, image)? {
                (diff, Cow::Owned(ruffle_name))
            } else {
                return Ok(());
            }
        }
    };

    // A comparison failed: write difference images and return an error.
    let env_name = render_interface.name();
    write_image(
        &base_path.join(format!("{name}.actual-{env_name}.png"))?,
        &actual_image,
    )?;
    write_image(
        &base_path.join(format!("{failure_name}.difference-color-{env_name}.png"))?,
        &failure.difference_color()?,
    )?;
    if let Some(diff_alpha) = failure.difference_alpha()? {
        write_image(
            &base_path.join(format!("{failure_name}.difference-alpha-{env_name}.png"))?,
            &diff_alpha,
        )?;
    }

    Err(anyhow!(
        "{failure_name} failed: \
        Number of outliers ({}) is bigger than allowed limit of {}. \
        Max difference is {}",
        failure.outliers,
        failure.max_outliers,
        failure.max_difference,
    ))
}

fn test(
    comparison: &ImageComparison,
    name: &str,
    actual_image: &image::RgbaImage,
    expected_image: image::RgbaImage,
) -> anyhow::Result<Option<ImageDiff>> {
    if actual_image.width() != expected_image.width()
        || actual_image.height() != expected_image.height()
    {
        return Err(anyhow!(
            "'{}' image is not the right size. Expected = {}x{}, actual = {}x{}.",
            name,
            expected_image.width(),
            expected_image.height(),
            actual_image.width(),
            actual_image.height()
        ));
    }

    let mut is_alpha_different = false;

    let difference_data: Vec<u8> =
        calculate_difference_data(actual_image, &expected_image, &mut is_alpha_different);

    let checks = comparison
        .checks()
        .map_err(|err| anyhow!("Image '{name}' failed: {err}"))?;

    let mut any_check_executed = false;
    for (i, check) in checks.iter().enumerate() {
        let check_name = format!("Image '{name}' check {i}");
        let filter_passed = check
            .filter
            .as_ref()
            .map(|f| f.evaluate())
            .unwrap_or(Ok(true))?;
        if !filter_passed {
            println!("{check_name} skipped: Filtered out.");
            continue;
        }

        let outliers = calculate_outliers(&difference_data, check.tolerance);
        let max_outliers = check.max_outliers;
        let max_difference = calculate_max_difference(&difference_data);

        any_check_executed = true;
        if outliers <= max_outliers {
            println!(
                "{check_name} succeeded: {outliers} outliers found, max difference {max_difference}"
            );
            continue;
        }

        // The image failed a check :(
        return Ok(Some(ImageDiff {
            width: actual_image.width(),
            height: actual_image.height(),
            difference_data,
            outliers,
            max_outliers,
            max_difference,
            is_alpha_different,
        }));
    }

    if !any_check_executed {
        return Err(anyhow!("Image '{name}' failed: No checks executed."));
    }

    Ok(None)
}

struct ImageDiff {
    width: u32,
    height: u32,
    difference_data: Vec<u8>,
    outliers: usize,
    max_outliers: usize,
    max_difference: u8,
    is_alpha_different: bool,
}

impl ImageDiff {
    fn difference_color(&self) -> anyhow::Result<image::RgbImage> {
        let mut difference_color =
            Vec::with_capacity(self.width as usize * self.height as usize * 3);
        for p in self.difference_data.chunks_exact(4) {
            difference_color.extend_from_slice(&p[..3]);
        }

        image::RgbImage::from_raw(self.width, self.height, difference_color)
            .context("Couldn't create color difference image")
    }

    fn difference_alpha(&self) -> anyhow::Result<Option<image::GrayImage>> {
        if self.is_alpha_different {
            let mut difference_alpha =
                Vec::with_capacity(self.width as usize * self.height as usize);
            for p in self.difference_data.chunks_exact(4) {
                difference_alpha.push(p[3])
            }

            image::GrayImage::from_raw(self.width, self.height, difference_alpha)
                .context("Couldn't create alpha difference image")
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

fn calculate_difference_data(
    actual_image: &image::RgbaImage,
    expected_image: &image::RgbaImage,
    is_alpha_different: &mut bool,
) -> Vec<u8> {
    expected_image
        .as_raw()
        .chunks_exact(4)
        .zip(actual_image.as_raw().chunks_exact(4))
        .flat_map(|(cmp_chunk, data_chunk)| {
            if cmp_chunk[3] != data_chunk[3] {
                *is_alpha_different = true;
            }

            [
                calc_difference(cmp_chunk[0], data_chunk[0]),
                calc_difference(cmp_chunk[1], data_chunk[1]),
                calc_difference(cmp_chunk[2], data_chunk[2]),
                calc_difference(cmp_chunk[3], data_chunk[3]),
            ]
        })
        .collect()
}

fn calculate_outliers(difference_data: &[u8], tolerance: u8) -> usize {
    difference_data
        .chunks_exact(4)
        .map(|colors| {
            (colors[0] > tolerance) as usize
                + (colors[1] > tolerance) as usize
                + (colors[2] > tolerance) as usize
                + (colors[3] > tolerance) as usize
        })
        .sum()
}

fn calculate_max_difference(difference_data: &[u8]) -> u8 {
    difference_data
        .chunks_exact(4)
        .map(|colors| colors[0].max(colors[1]).max(colors[2]).max(colors[3]))
        .max()
        .unwrap()
}

fn calc_difference(lhs: u8, rhs: u8) -> u8 {
    (lhs as i16 - rhs as i16).unsigned_abs() as u8
}

fn write_image<P, Container>(
    path: &VfsPath,
    image: &ImageBuffer<P, Container>,
) -> anyhow::Result<()>
where
    P: Pixel + PixelWithColorType,
    [P::Subpixel]: EncodableLayout,
    Container: Deref<Target = [P::Subpixel]>,
{
    let mut buffer = vec![];
    image.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)?;
    write_bytes(path, &buffer)?;
    Ok(())
}
