//! Ruffle benchmark suite
//!
//! `criterion` is used to run a benchmark for each directory containing a `bench.swf` in the
//! `benches/swfs` directory.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use tests::*;

fn bench_dir(c: &mut Criterion, directory: &str, bench_fn: fn(&mut TestPlayer)) {
    let mut group = c.benchmark_group(directory);
    let bench_dir = format!("benches/swfs/{}", directory);
    let dir_iter =
        std::fs::read_dir(&bench_dir).expect("Unable to open benches/swfs/avm1 directory");
    for entry in dir_iter {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if metadata.is_dir() {
            let file_name = entry.file_name();
            let bench_name = file_name.to_string_lossy();
            group.bench_function(bench_name.as_ref(), |b| {
                b.iter_batched_ref(
                    || {
                        TestBuilder::new()
                            .with_swf_path(&format!("{}/{}/bench.swf", bench_dir, bench_name))
                            .is_avm_bench(true)
                            .build()
                            .expect("Unable to build player")
                    },
                    bench_fn,
                    BatchSize::PerIteration,
                )
            });
        }
    }
}

pub fn avm1_benches(c: &mut Criterion) {
    bench_dir(c, "avm1", TestPlayer::run_avm1_bench);
}

pub fn avm2_benches(c: &mut Criterion) {
    bench_dir(c, "avm2", TestPlayer::run_avm2_bench);
}

criterion_group!(benches, avm1_benches, avm2_benches);
criterion_main!(benches);
