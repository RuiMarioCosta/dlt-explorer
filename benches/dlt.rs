mod operations;

use criterion::{Criterion, criterion_group, criterion_main};
use operations::fixtures::BenchmarkProfile;

fn bench_operations(c: &mut Criterion) {
    let profile = BenchmarkProfile::from_env();

    operations::open_file::bench(c, profile);
}

criterion_group!(benches, bench_operations);
criterion_main!(benches);
