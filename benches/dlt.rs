use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use dlt_explorer::dlt::v1::framer as v1_framer;
use dlt_explorer::dlt::v2::framer as v2_framer;
use dlt_explorer::dlt::v2::Dlt;
use dlt_explorer::dlt::v2::test_helpers::V2MessageBuilder;
use std::hint::black_box;
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

fn minimal_v1_frame_with_storage_ecu(storage_ecu: &[u8; 4]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"DLT\x01");
    buf.extend_from_slice(&100u32.to_le_bytes());
    buf.extend_from_slice(&500u32.to_le_bytes());
    buf.extend_from_slice(storage_ecu);

    let htyp: u8 = 1 << 5;
    buf.push(htyp);
    buf.push(0);
    let len: u16 = 4;
    buf.extend_from_slice(&len.to_be_bytes());
    buf
}

fn generate_v1_scan_data(count: usize, mixed_every: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(count * 20);
    for i in 0..count {
        let ecu = if mixed_every > 0 && i % mixed_every == 0 {
            b"ECU2"
        } else {
            b"ECU1"
        };
        data.extend_from_slice(&minimal_v1_frame_with_storage_ecu(ecu));
    }
    data
}

fn generate_v2_scan_data(count: usize, mixed_every: usize) -> Vec<u8> {
    let mut data = Vec::new();
    for i in 0..count {
        let storage_ecu = if mixed_every > 0 && i % mixed_every == 0 {
            *b"ECU2"
        } else {
            *b"ECU1"
        };
        let msg = V2MessageBuilder::new()
            .with_storage_ecu(&storage_ecu)
            .with_verbose_string("scan bench payload")
            .build();
        data.extend_from_slice(&msg);
    }
    data
}

fn bench_scan_frames(c: &mut Criterion) {
    let count = 100_000usize;
    let v1_uniform = generate_v1_scan_data(count, 0);
    let v1_sparse_mixed = generate_v1_scan_data(count, 1000);
    let v2_uniform = generate_v2_scan_data(count, 0);
    let v2_sparse_mixed = generate_v2_scan_data(count, 1000);

    let mut group = c.benchmark_group("scan_frames_ecu");
    group.sample_size(50);
    group.throughput(Throughput::Elements(count as u64));

    group.bench_function(BenchmarkId::new("v1_scan", "uniform_ecu"), |b| {
        b.iter(|| black_box(v1_framer::scan_frames(&v1_uniform, 0)));
    });

    group.bench_function(BenchmarkId::new("v1_scan", "sparse_mixed_ecu"), |b| {
        b.iter(|| black_box(v1_framer::scan_frames(&v1_sparse_mixed, 0)));
    });

    group.bench_function(BenchmarkId::new("v2_scan", "uniform_ecu"), |b| {
        b.iter(|| black_box(v2_framer::scan_frames(&v2_uniform, 0)));
    });

    group.bench_function(BenchmarkId::new("v2_scan", "sparse_mixed_ecu"), |b| {
        b.iter(|| black_box(v2_framer::scan_frames(&v2_sparse_mixed, 0)));
    });

    group.finish();
}

criterion_group!(benches, bench_v2_open, bench_scan_frames);
criterion_main!(benches);
