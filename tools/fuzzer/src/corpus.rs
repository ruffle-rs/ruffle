use anyhow::anyhow;
use std::{
    path::Path,
    path::PathBuf,
    process::{Command, Output},
};

use crate::fuzz_dir;

pub fn corpus_dir() -> PathBuf {
    fuzz_dir().join("corpus")
}

fn git(dir: &Path, args: &[&str]) -> std::io::Result<Output> {
    Command::new("git").args(args).current_dir(dir).output()
}

pub fn prepare_corpus() -> anyhow::Result<()> {
    prepare_corpus_repo()?;
    Ok(())
}

fn prepare_corpus_repo() -> anyhow::Result<()> {
    let corpus_dir = corpus_dir();

    if !corpus_dir.join(".git").exists() {
        if corpus_dir.exists() {
            return Err(anyhow!(
                "The corpus directory exists but is not a git repository"
            ));
        }

        let output = Command::new("git")
            .args([
                "clone",
                "https://github.com/ruffle-rs/fuzz-corpus.git",
                corpus_dir.to_str().unwrap(),
            ])
            .output()?;
        if !output.status.success() {
            eprintln!("git stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("git stdout:\n{}", String::from_utf8_lossy(&output.stdout));
            return Err(anyhow!("Failed to clone the corpus"));
        }
    }

    let has_upstream = git(
        &corpus_dir,
        &["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
    )?
    .status
    .success();
    if !has_upstream {
        eprintln!("Warning: The corpus repository has no upstream configured; skipping pull");
        return Ok(());
    }

    let clean = git(&corpus_dir, &["status", "--porcelain"])?
        .stdout
        .is_empty();
    if !clean {
        eprintln!("Warning: The corpus repository has local changes; skipping pull");
        return Ok(());
    }

    let ahead = git(&corpus_dir, &["rev-list", "--count", "@{u}..HEAD"])?;
    let ahead: u32 = String::from_utf8_lossy(&ahead.stdout)
        .trim()
        .parse()
        .unwrap_or(0);
    if ahead > 0 {
        eprintln!("Warning: The corpus repository has {ahead} local commit(s); skipping pull");
        return Ok(());
    }

    let head_before = git(&corpus_dir, &["rev-parse", "HEAD"])?;
    let head_before = String::from_utf8_lossy(&head_before.stdout)
        .trim()
        .to_owned();

    let pull = git(&corpus_dir, &["pull"])?;
    if !pull.status.success() {
        return Err(anyhow!(
            "Failed to pull the corpus:\n{}{}",
            String::from_utf8_lossy(&pull.stdout),
            String::from_utf8_lossy(&pull.stderr),
        ));
    }

    let pulled = git(
        &corpus_dir,
        &["rev-list", "--count", &format!("{head_before}..HEAD")],
    )?;
    let pulled: u32 = String::from_utf8_lossy(&pulled.stdout)
        .trim()
        .parse()
        .unwrap_or(0);
    eprintln!("Pulled {pulled} commit(s) into the corpus");

    Ok(())
}
