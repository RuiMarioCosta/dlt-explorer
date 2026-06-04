use criterion::{BenchmarkId, Criterion};
use dlt_explorer::dlt::protocol::CNTI_VERBOSE;
use dlt_explorer::dlt::v1::payload as v1_payload;
use dlt_explorer::dlt::v2::payload as v2_payload;
use std::hint::black_box;

use super::fixtures::{build_v1_verbose_payload, payload_text, scenarios_for_profile, BenchmarkProfile};

pub fn bench(c: &mut Criterion, profile: BenchmarkProfile) {
    let mut group = c.benchmark_group("decode_payload");

    for &spec in scenarios_for_profile(profile) {
        let mut raw = build_v1_verbose_payload(payload_text(spec));
        if spec.truncated_tail && !raw.is_empty() {
            raw.pop();
        }

        let htyp_v1_verbose_be: u8 = (1 << 5) | 0x01 | 0x02;
        let msin_v1_verbose: u8 = 0x01;

        group.bench_function(BenchmarkId::new("v1_decode_payload", spec.name), |b| {
            b.iter(|| black_box(v1_payload::decode_payload(htyp_v1_verbose_be, msin_v1_verbose, &raw)));
        });

        group.bench_function(BenchmarkId::new("v2_decode_payload", spec.name), |b| {
            b.iter(|| black_box(v2_payload::decode_payload(CNTI_VERBOSE, &raw)));
        });
    }

    group.finish();
}
