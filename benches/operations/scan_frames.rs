use criterion::{BenchmarkId, Criterion, Throughput};
use dlt_explorer::dlt::v1::framer as v1_framer;
use dlt_explorer::dlt::v2::framer as v2_framer;
use std::hint::black_box;

use super::fixtures::{build_v1_dataset, build_v2_dataset, scenarios_for_profile, BenchmarkProfile};

pub fn bench(c: &mut Criterion, profile: BenchmarkProfile) {
    let mut group = c.benchmark_group("scan_frames");

    for &spec in scenarios_for_profile(profile) {
        let v1_data = build_v1_dataset(spec);
        let v2_data = build_v2_dataset(spec);

        group.throughput(Throughput::Elements(spec.count as u64));

        group.bench_function(BenchmarkId::new("v1_scan_frames", spec.name), |b| {
            b.iter(|| black_box(v1_framer::scan_frames(&v1_data, 0)));
        });

        group.bench_function(BenchmarkId::new("v2_scan_frames", spec.name), |b| {
            b.iter(|| black_box(v2_framer::scan_frames(&v2_data, 0)));
        });
    }

    group.finish();
}
