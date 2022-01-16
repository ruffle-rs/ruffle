//! Ruffle benchmark suite
//!
//! `criterion` is used to run a benchmark for each directory containing a `bench.swf` in the
//! `benches/swfs` directory.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::time::Duration;
use tests::*;

pub fn avm1_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("avm1");
    group.measurement_time(Duration::from_secs(10));

    // Iterate over every directory in the `benches/swfs`.
    let bench_dir =
        std::fs::read_dir("benches/swfs/avm1").expect("Unable to open benches/swfs/avm1 directory");
    for entry in bench_dir {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if metadata.is_dir() {
            let file_name = entry.file_name();
            let bench_name = file_name.to_string_lossy();
            group.bench_function(bench_name.as_ref(), |b| {
                b.iter_batched_ref(
                    || {
                        TestBuilder::new()
                            .with_swf_path(&format!("benches/swfs/avm1/{}/bench.swf", bench_name))
                            .is_avm_bench(true)
                            .build()
                            .expect("Unable to build player")
                    },
                    |player| player.run_avm1_bench(),
                    BatchSize::PerIteration,
                )
            });
        }
    }
}

criterion_group!(benches, avm1_benches);
criterion_main!(benches);
