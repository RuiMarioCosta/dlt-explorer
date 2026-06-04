use criterion::{BenchmarkId, Criterion};
use dlt_explorer::dlt::v1::header::parse_v1_header;
use dlt_explorer::dlt::v2::header::parse_v2_header;
use std::hint::black_box;

use super::fixtures::{SHARED_SCENARIOS, build_v1_dataset, build_v2_dataset};

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_header");

    for spec in SHARED_SCENARIOS {
        let v1_data = build_v1_dataset(spec);
        let v2_data = build_v2_dataset(spec);

        let v1_msg = if v1_data.len() > 16 {
            &v1_data[16..]
        } else {
            &v1_data[..]
        };
        let v2_msg = if v2_data.len() > 16 {
            &v2_data[16..]
        } else {
            &v2_data[..]
        };

        group.bench_function(BenchmarkId::new("v1_parse_header", spec.name), |b| {
            b.iter(|| black_box(parse_v1_header(v1_msg)));
        });

        group.bench_function(BenchmarkId::new("v2_parse_header", spec.name), |b| {
            b.iter(|| black_box(parse_v2_header(v2_msg)));
        });
    }

    group.finish();
}
