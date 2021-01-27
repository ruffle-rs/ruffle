use criterion::{criterion_group, criterion_main, Criterion};
use swf::test_data;

pub fn bench_read_tag(c: &mut Criterion) {
    let mut group = c.benchmark_group("read-tag");

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

    for (i, data) in test_data::avm2_tests().iter().enumerate() {
        group.bench_with_input(i.to_string(), data, |b, (_, _, bytes)| {
            b.iter_with_large_drop(|| {
                let mut reader = swf::avm2::read::Reader::new(&bytes[..]);
                reader.read()
            })
        });
    }
}

criterion_group!(read, bench_read_tag, bench_avm1_read, bench_avm2_read);
criterion_main!(read);
