use criterion::{BenchmarkId, Criterion, Throughput};
use dlt_explorer::dlt::v1::Dlt as V1Dlt;
use dlt_explorer::dlt::v2::Dlt as V2Dlt;
use std::fs;

use super::fixtures::{build_v1_dataset, build_v2_dataset, scenarios_for_profile, BenchmarkProfile};

pub fn bench(c: &mut Criterion, profile: BenchmarkProfile) {
    let tempdir = tempfile::tempdir().expect("create open_file tempdir");
    let mut group = c.benchmark_group("open_file");

    for &spec in scenarios_for_profile(profile) {
        let v1_data = build_v1_dataset(spec);
        let v2_data = build_v2_dataset(spec);

        let v1_path = tempdir.path().join(format!("v1_{}.dlt", spec.name));
        let v2_path = tempdir.path().join(format!("v2_{}.dlt", spec.name));

        fs::write(&v1_path, &v1_data).expect("write v1 open_file fixture");
        fs::write(&v2_path, &v2_data).expect("write v2 open_file fixture");

        group.throughput(Throughput::Bytes(v1_data.len() as u64));
        group.bench_function(BenchmarkId::new("v1_open_file", spec.name), |b| {
            b.iter(|| V1Dlt::open(vec![v1_path.clone()]).unwrap());
        });

        group.throughput(Throughput::Bytes(v2_data.len() as u64));
        group.bench_function(BenchmarkId::new("v2_open_file", spec.name), |b| {
            b.iter(|| V2Dlt::open(vec![v2_path.clone()]).unwrap());
        });
    }

    group.finish();
}
