use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dlt_explorer::dlt::Dlt;
use dlt_explorer::dlt::test_helpers::V2MessageBuilder;
use std::io::Write;
use std::path::PathBuf;

/// Generate a synthetic v2 DLT file with `count` messages, returning its path
/// and byte size.
fn generate_v2_file(dir: &std::path::Path, count: usize) -> (PathBuf, u64) {
    let path = dir.join("bench_100k.dlt");
    let mut f = std::fs::File::create(&path).expect("create bench file");
    for i in 0..count {
        let msg = V2MessageBuilder::new()
            .with_apid("BNCH")
            .with_ctid("PERF")
            .with_ecu("ECU1")
            .with_storage_timestamp(1000 + (i as u32 / 1000), (i as u32 % 1000) * 1_000_000)
            .with_timestamp_ns(i as u64 * 1_000_000)
            .with_verbose_string("benchmark payload data")
            .build();
        f.write_all(&msg).expect("write bench message");
    }
    f.flush().expect("flush bench file");
    let size = path.metadata().expect("bench file metadata").len();
    (path, size)
}

fn bench_v2_open(c: &mut Criterion) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let (path, file_size) = generate_v2_file(dir.path(), 100_000);

    let mut group = c.benchmark_group("dlt_parsing");
    group.throughput(Throughput::Bytes(file_size));
    group.bench_function(BenchmarkId::new("v2_open", "100k_rows"), |b| {
        b.iter(|| Dlt::open(vec![path.clone()]).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_v2_open);
criterion_main!(benches);
