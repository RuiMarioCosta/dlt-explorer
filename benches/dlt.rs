mod operations;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_operations(c: &mut Criterion) {
    operations::scan_frames::bench(c);
    operations::parse_header::bench(c);
    operations::decode_payload::bench(c);
    operations::open_file::bench(c);
}

criterion_group!(benches, bench_operations);
criterion_main!(benches);
