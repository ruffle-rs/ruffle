//! Lifetime tests for the libraries of dynamically loaded movies.

use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::thread::sleep;

/// A SWF that has been loaded and unloaded must not leave its library behind.
///
/// Ruffle stores one library per movie in a map that is weakly keyed on the
/// movie, but a library's characters hold strong `Arc<SwfMovie>` clones of that
/// same movie, so the key's strong count never falls to zero on its own. Before
/// this was handled, every SWF a movie loaded stayed resident - with all of its
/// characters and decoded bitmaps - for the rest of the session.
///
/// The test movie loads the same child ten times over, unloading each one
/// before the next, so afterwards none of them can be reached from the display
/// list or from ActionScript.
/// How many load/unload cycles `test.swf` performs.
const CYCLES: usize = 10;

pub fn loader_unload_releases_library(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(150),
            output_path: "output.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new(
            "tests/swfs/avm2/loader_unload_releases_library/",
        )),
        "loader_unload_releases_library".to_string(),
    )?;

    let mut runner = test.create_test_runner(environment)?;
    loop {
        match runner.tick()? {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    let mut player = runner.player().lock().unwrap();
    let resident: Vec<String> = player.mutate_with_update_context(|context| {
        context
            .library
            .known_movies()
            .map(|movie| movie.url().to_owned())
            .collect()
    });

    let children: Vec<&String> = resident
        .iter()
        .filter(|url| url.ends_with("child.swf"))
        .collect();

    // The tail of the run is allowed: the last cycle or two may not have been
    // through a collection yet, so a small constant can still be resident. What
    // must not happen is retention that grows with the number of loads, which
    // is what leaves all ten of them here.
    const ALLOWED: usize = 2;
    if children.len() > ALLOWED {
        return Err(format!(
            "{} of the {CYCLES} unloaded child movies are still resident, expected at most \
             {ALLOWED}: {children:?}",
            children.len(),
        )
        .into());
    }

    Ok(())
}
