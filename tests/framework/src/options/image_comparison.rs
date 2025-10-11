use crate::image_trigger::ImageTrigger;
use crate::options::TestExpression;
use crate::util::write_image;
use anyhow::anyhow;
use image::ImageFormat;
use serde::Deserialize;
use std::borrow::Cow;
use vfs::VfsPath;

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct ImageComparison {
    tolerance: Option<u8>,
    max_outliers: Option<usize>,
    checks: Vec<ImageComparisonCheck>,
    pub trigger: ImageTrigger,
}

fn calc_difference(lhs: u8, rhs: u8) -> u8 {
    (lhs as i16 - rhs as i16).unsigned_abs() as u8
}

impl ImageComparison {
    fn checks(&self) -> anyhow::Result<Cow<'_, [ImageComparisonCheck]>> {
        let has_simple_check = self.tolerance.is_some() || self.max_outliers.is_some();
        if has_simple_check && !self.checks.is_empty() {
            return Err(anyhow!(
                "Both simple and advanced checks are defined. \
                Either remove 'tolerance' & 'max_outliers', or move it to 'checks'."
            ));
        }

        if !self.checks.is_empty() {
            Ok(Cow::Borrowed(&self.checks))
        } else {
            Ok(Cow::Owned(vec![ImageComparisonCheck {
                tolerance: self.tolerance.unwrap_or_default(),
                max_outliers: self.max_outliers.unwrap_or_default(),
                filter: None,
            }]))
        }
    }

    pub fn test(
        &self,
        name: &str,
        actual_image: image::RgbaImage,
        expected_image: image::RgbaImage,
        test_path: &VfsPath,
        environment_name: String,
        known_failure: bool,
    ) -> anyhow::Result<()> {
        use anyhow::Context;

        let save_actual_image = || {
            if !known_failure {
                // If we're expecting failure, spamming files isn't productive.
                write_image(
                    &test_path.join(format!("{name}.actual-{environment_name}.png"))?,
                    &actual_image,
                    ImageFormat::Png,
                )
            } else {
                Ok(())
            }
        };

        if actual_image.width() != expected_image.width()
            || actual_image.height() != expected_image.height()
        {
            save_actual_image()?;
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

        let difference_data: Vec<u8> = Self::calculate_difference_data(
            &actual_image,
            &expected_image,
            &mut is_alpha_different,
        );

        let checks = self
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

            let outliers = Self::calculate_outliers(&difference_data, check.tolerance);
            let max_outliers = check.max_outliers;
            let max_difference = Self::calculate_max_difference(&difference_data);

            any_check_executed = true;
            if outliers <= max_outliers {
                println!("{check_name} succeeded: {outliers} outliers found, max difference {max_difference}");
                continue;
            }

            // The image failed a check :(

            save_actual_image()?;

            let mut difference_color = Vec::with_capacity(
                actual_image.width() as usize * actual_image.height() as usize * 3,
            );
            for p in difference_data.chunks_exact(4) {
                difference_color.extend_from_slice(&p[..3]);
            }

            if !known_failure {
                // If we're expecting failure, spamming files isn't productive.
                let difference_image = image::RgbImage::from_raw(
                    actual_image.width(),
                    actual_image.height(),
                    difference_color,
                )
                .context("Couldn't create color difference image")?;
                write_image(
                    &test_path.join(format!("{name}.difference-color-{environment_name}.png"))?,
                    &difference_image,
                    ImageFormat::Png,
                )?;
            }

            if is_alpha_different {
                let mut difference_alpha = Vec::with_capacity(
                    actual_image.width() as usize * actual_image.height() as usize,
                );
                for p in difference_data.chunks_exact(4) {
                    difference_alpha.push(p[3])
                }

                if !known_failure {
                    // If we're expecting failure, spamming files isn't productive.
                    let difference_image = image::GrayImage::from_raw(
                        actual_image.width(),
                        actual_image.height(),
                        difference_alpha,
                    )
                    .context("Couldn't create alpha difference image")?;
                    write_image(
                        &test_path
                            .join(format!("{name}.difference-alpha-{environment_name}.png"))?,
                        &difference_image,
                        ImageFormat::Png,
                    )?;
                }
            }

            return Err(anyhow!(
                "{check_name} failed: \
                Number of outliers ({outliers}) is bigger than allowed limit of {max_outliers}. \
                Max difference is {max_difference}",
            ));
        }

        if !any_check_executed {
            return Err(anyhow!("Image '{name}' failed: No checks executed.",));
        }

        Ok(())
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
}

#[derive(Deserialize, Default, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
struct ImageComparisonCheck {
    tolerance: u8,
    max_outliers: usize,

    filter: Option<TestExpression>,
}
