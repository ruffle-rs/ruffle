use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::ffi::OsStr;
use std::fs;
use std::time::Duration;
use swf::test_data;

pub fn bench_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read");
    group.measurement_time(Duration::from_secs(10));
    group.noise_threshold(0.02);
    group.confidence_level(0.98);

    for entry in fs::read_dir("benches/real-world-swfs").unwrap() {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("swf")) {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let data = fs::read(&path).unwrap();
                let swf_buf = swf::decompress_swf(&*data).unwrap();
                group.bench_with_input(file_name, &swf_buf, |b, swf_buf| {
                    b.iter_with_large_drop(|| {
                        let swf = swf::parse_swf(swf_buf).unwrap();

                        // If we make some of these contain iterables in the future, iterate through them here!
                        swf.tags.iter().for_each(|tag| {
                            match tag {
                                _ => black_box(tag),
                            };
                        });

                        swf
                    })
                });
            }
        }
    }
}

pub fn bench_read_tag(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-tag");
    group.noise_threshold(0.02);

    for (i, data) in test_data::tag_tests().iter().enumerate() {
        group.bench_with_input(i.to_string(), data, |b, (swf_version, _, tag_bytes)| {
            b.iter_with_large_drop(|| {
                let mut reader = swf::read::Reader::new(&tag_bytes[..], *swf_version);
                reader.read_tag()
            })
        });
    }
}

pub fn bench_avm1_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-avm1");
    group.noise_threshold(0.02);

    for (i, data) in test_data::avm1_tests().iter().enumerate() {
        group.bench_with_input(i.to_string(), data, |b, (swf_version, _, bytes)| {
            b.iter_with_large_drop(|| {
                let mut reader = swf::avm1::read::Reader::new(&bytes[..], *swf_version);
                reader.read_action()
            })
        });
    }
}

pub fn bench_avm2_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-avm2");
    group.noise_threshold(0.02);

    for (i, data) in test_data::avm2_tests().iter().enumerate() {
        group.bench_with_input(i.to_string(), data, |b, (_, _, bytes)| {
            b.iter_with_large_drop(|| {
                let mut reader = swf::avm2::read::Reader::new(&bytes[..]);
                reader.read()
            })
        });
    }
}

criterion_group!(
    read,
    bench_read,
    bench_read_tag,
    bench_avm1_read,
    bench_avm2_read
);
criterion_main!(read);
