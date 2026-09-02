//! Lifetime tests for the libraries of dynamically loaded movies.

use ruffle_core::Player;
use ruffle_test_framework::environment::Environment;
use ruffle_test_framework::options::TestOptions;
use ruffle_test_framework::runner::TestStatus;
use ruffle_test_framework::test::Test;
use ruffle_test_framework::vfs::{PhysicalFS, VfsPath};
use std::thread::sleep;

/// The movies the player currently holds a library for, by URL.
fn resident_movies(player: &mut Player) -> Vec<String> {
    player.mutate_with_update_context(|context| {
        context
            .library
            .known_movies()
            .map(|movie| movie.url().to_owned())
            .collect()
    })
}

fn resident_children(player: &mut Player) -> Vec<String> {
    resident_movies(player)
        .into_iter()
        .filter(|url| url.ends_with("child.swf"))
        .collect()
}

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
    let children = resident_children(&mut player);

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

/// Runs `loader_unload_retains_linked_class` and, `frame` frames in, forces
/// a full collection and hands the player to `check`. The run then continues
/// to the end, where the movie's trace output is compared as usual.
///
/// The movie loads a child SWF into a child `ApplicationDomain`, keeps a
/// class from that domain, unloads and drops the content, instantiates the
/// class at frame 90, and releases the class and domain at frame 150.
fn run_linked_class_test(
    environment: &impl Environment,
    frame: u32,
    check: impl FnOnce(&mut Player) -> Result<(), String>,
) -> Result<(), libtest_mimic::Failed> {
    let test = &Test::from_options(
        TestOptions {
            num_frames: Some(240),
            output_path: "output.txt".into(),
            ..Default::default()
        },
        VfsPath::new(PhysicalFS::new(
            "tests/swfs/avm2/loader_unload_retains_linked_class/",
        )),
        "loader_unload_retains_linked_class".to_string(),
    )?;

    let mut runner = test.create_test_runner(environment)?;
    let mut frames = 0;
    let mut check = Some(check);
    loop {
        let status = runner.tick()?;
        if runner.is_preloaded() {
            frames += 1;
        }
        if frames == frame
            && let Some(check) = check.take()
        {
            let mut player = runner.player().lock().unwrap();
            // Two full cycles: the first may only get as far as dropping a
            // library, whose objects are then swept by the second.
            player.collect_all_garbage();
            player.collect_all_garbage();
            check(&mut player)?;
        }
        match status {
            TestStatus::Continue => {}
            TestStatus::Sleep(duration) => sleep(duration),
            TestStatus::Finished => break,
        }
    }

    if check.is_some() {
        return Err(format!("the movie never reached frame {frame}").into());
    }
    Ok(())
}

/// A class taken out of a loaded SWF's `ApplicationDomain` keeps that SWF's
/// library alive after the content has been unloaded and collected - the
/// class can still be instantiated, and instantiating it still finds the
/// characters it is linked to.
///
/// The instantiation itself is checked by the movie's trace output; this
/// checks the library from the outside, after a full collection, at a point
/// where the content is long gone but the class is still held.
pub fn retained_class_keeps_library(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    run_linked_class_test(environment, 60, |player| {
        let children = resident_children(player);
        if children.len() != 1 {
            return Err(format!(
                "expected the child movie to still be resident while its class is held, \
                 found {children:?}"
            ));
        }
        Ok(())
    })
}

/// Once the class and the domain are released as well, nothing needs the
/// loaded SWF any more and its library goes away.
pub fn released_class_frees_library(
    environment: &impl Environment,
) -> Result<(), libtest_mimic::Failed> {
    run_linked_class_test(environment, 200, |player| {
        let children = resident_children(player);
        if !children.is_empty() {
            return Err(format!(
                "expected the child movie to be released once its class was, found {children:?}"
            ));
        }
        Ok(())
    })
}
