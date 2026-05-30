use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dlt_explorer::dlt::v1::Dlt;
use std::path::PathBuf;

fn bench_from_files(c: &mut Criterion) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/data/testfile_100k_rows.dlt");
    let file_size = path.metadata().expect("test file must exist").len();

    let mut group = c.benchmark_group("dlt_parsing");
    group.throughput(Throughput::Bytes(file_size));
    group.bench_function(BenchmarkId::new("from_files", "100k_rows"), |b| {
        b.iter(|| Dlt::from_files(vec![path.clone()], None).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_from_files);
criterion_main!(benches);
