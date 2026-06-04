mod operations;

use criterion::{Criterion, criterion_group, criterion_main};
use operations::fixtures::BenchmarkProfile;

fn bench_operations(c: &mut Criterion) {
    let profile = BenchmarkProfile::from_env();

    operations::scan_frames::bench(c, profile);
    operations::parse_header::bench(c, profile);
    operations::decode_payload::bench(c, profile);
    operations::open_file::bench(c, profile);
}

criterion_group!(benches, bench_operations);
criterion_main!(benches);
