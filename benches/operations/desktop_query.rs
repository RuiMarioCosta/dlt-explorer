use criterion::{BenchmarkId, Criterion};
use dlt_explorer::desktop::DesktopBenchmarkHarness;
use std::fs;
use std::hint::black_box;

use super::fixtures::{
    BenchmarkProfile, build_v1_dataset, build_v2_dataset, scenarios_for_profile,
};

const VIEWPORT_ROWS: usize = 128;

pub fn bench(c: &mut Criterion, profile: BenchmarkProfile) {
    let tempdir = tempfile::tempdir().expect("create desktop_query tempdir");
    {
        let mut query_group = c.benchmark_group("desktop_query_update");

        for &spec in scenarios_for_profile(profile) {
            let v1_data = build_v1_dataset(spec);
            let v2_data = build_v2_dataset(spec);

            let v1_path = tempdir.path().join(format!("v1_{}.dlt", spec.name));
            let v2_path = tempdir.path().join(format!("v2_{}.dlt", spec.name));

            fs::write(&v1_path, &v1_data).expect("write v1 desktop fixture");
            fs::write(&v2_path, &v2_data).expect("write v2 desktop fixture");

            let mut v1_query_harness =
                DesktopBenchmarkHarness::load(vec![v1_path.clone()]).expect("load v1 dataset");
            v1_query_harness.set_kind_filter_contains("log");
            let v1_query_token = v1_query_harness
                .first_visible_timestamp()
                .unwrap_or_else(|| "0.000000".to_string());
            let mut v1_query_toggle = false;
            query_group.bench_function(
                BenchmarkId::new("v1_desktop_query_update", spec.name),
                move |b| {
                    b.iter(|| {
                        v1_query_toggle = !v1_query_toggle;
                        if v1_query_toggle {
                            v1_query_harness
                                .set_rendered_search_query(v1_query_token.as_str());
                        } else {
                            v1_query_harness
                                .set_rendered_search_query("no-such-rendered-text-token");
                        }
                        black_box(v1_query_harness.visible_message_count());
                    });
                },
            );

            let mut v2_query_harness =
                DesktopBenchmarkHarness::load(vec![v2_path]).expect("load v2 dataset");
            v2_query_harness.set_kind_filter_contains("log");
            let v2_query_token = v2_query_harness
                .first_visible_timestamp()
                .unwrap_or_else(|| "0.000000".to_string());
            let mut v2_query_toggle = false;
            query_group.bench_function(
                BenchmarkId::new("v2_desktop_query_update", spec.name),
                move |b| {
                    b.iter(|| {
                        v2_query_toggle = !v2_query_toggle;
                        if v2_query_toggle {
                            v2_query_harness
                                .set_rendered_search_query(v2_query_token.as_str());
                        } else {
                            v2_query_harness
                                .set_rendered_search_query("no-such-rendered-text-token");
                        }
                        black_box(v2_query_harness.visible_message_count());
                    });
                },
            );
        }

        query_group.finish();
    }

    {
        let mut viewport_group = c.benchmark_group("log_table_viewport");

        for &spec in scenarios_for_profile(profile) {
            let v1_data = build_v1_dataset(spec);
            let v2_data = build_v2_dataset(spec);

            let v1_path = tempdir.path().join(format!("v1_{}.dlt", spec.name));
            let v2_path = tempdir.path().join(format!("v2_{}.dlt", spec.name));

            fs::write(&v1_path, &v1_data).expect("write v1 desktop fixture");
            fs::write(&v2_path, &v2_data).expect("write v2 desktop fixture");

            let mut v1_viewport_harness =
                DesktopBenchmarkHarness::load(vec![v1_path.clone()]).expect("load v1 dataset");
            v1_viewport_harness.set_kind_filter_contains("log");
            v1_viewport_harness.set_rendered_search_query("payload");
            let mut v1_cursor = 0usize;
            viewport_group.bench_function(
                BenchmarkId::new("v1_log_table_viewport", spec.name),
                move |b| {
                    b.iter(|| {
                        let total_rows = v1_viewport_harness.visible_message_count();
                        let start = if total_rows > VIEWPORT_ROWS {
                            v1_cursor % (total_rows - VIEWPORT_ROWS + 1)
                        } else {
                            0
                        };
                        let consumed =
                            v1_viewport_harness.read_visible_row_window(start, VIEWPORT_ROWS);
                        v1_cursor = v1_cursor.wrapping_add(31);
                        black_box(consumed);
                    });
                },
            );

            let mut v2_viewport_harness =
                DesktopBenchmarkHarness::load(vec![v2_path]).expect("load v2 dataset");
            v2_viewport_harness.set_kind_filter_contains("log");
            v2_viewport_harness.set_rendered_search_query("payload");
            let mut v2_cursor = 0usize;
            viewport_group.bench_function(
                BenchmarkId::new("v2_log_table_viewport", spec.name),
                move |b| {
                    b.iter(|| {
                        let total_rows = v2_viewport_harness.visible_message_count();
                        let start = if total_rows > VIEWPORT_ROWS {
                            v2_cursor % (total_rows - VIEWPORT_ROWS + 1)
                        } else {
                            0
                        };
                        let consumed =
                            v2_viewport_harness.read_visible_row_window(start, VIEWPORT_ROWS);
                        v2_cursor = v2_cursor.wrapping_add(31);
                        black_box(consumed);
                    });
                },
            );
        }

        viewport_group.finish();
    }
}
