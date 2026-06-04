use dlt_explorer::dlt::payload::{DLT_SCOD_UTF8, DLT_TYPE_INFO_STRG};
use dlt_explorer::dlt::v2::test_helpers::V2MessageBuilder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenchmarkSizes {
    pub small: usize,
    pub mixed: usize,
    pub large: usize,
}

pub const BENCHMARK_SIZES: BenchmarkSizes = BenchmarkSizes {
    small: 1_000,
    mixed: 5_000,
    large: 100_000,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenchmarkProfile {
    Smoke,
    Full,
}

impl BenchmarkProfile {
    pub fn from_env() -> Self {
        match std::env::var("DLT_BENCH_PROFILE")
            .or_else(|_| std::env::var("BENCH_PROFILE"))
            .ok()
            .as_deref()
        {
            Some("smoke") => Self::Smoke,
            Some("full") | None => Self::Full,
            Some(other) => panic!(
                "unsupported benchmark profile '{other}'; expected 'smoke' or 'full'"
            ),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScenarioSpec {
    pub name: &'static str,
    pub count: usize,
    pub mixed_every: usize,
    pub marker_in_payload: bool,
    pub truncated_tail: bool,
}

pub const SHARED_SCENARIOS: [ScenarioSpec; 6] = [
    ScenarioSpec {
        name: "uniform_ecu_small",
        count: BENCHMARK_SIZES.small,
        mixed_every: 0,
        marker_in_payload: false,
        truncated_tail: false,
    },
    ScenarioSpec {
        name: "uniform_ecu_large",
        count: BENCHMARK_SIZES.large,
        mixed_every: 0,
        marker_in_payload: false,
        truncated_tail: false,
    },
    ScenarioSpec {
        name: "sparse_mixed_ecu_large",
        count: BENCHMARK_SIZES.large,
        mixed_every: 1_000,
        marker_in_payload: false,
        truncated_tail: false,
    },
    ScenarioSpec {
        name: "dense_mixed_ecu_large",
        count: BENCHMARK_SIZES.large,
        mixed_every: 2,
        marker_in_payload: false,
        truncated_tail: false,
    },
    ScenarioSpec {
        name: "marker_in_payload",
        count: BENCHMARK_SIZES.mixed,
        mixed_every: 0,
        marker_in_payload: true,
        truncated_tail: false,
    },
    ScenarioSpec {
        name: "truncated_tail",
        count: BENCHMARK_SIZES.mixed,
        mixed_every: 0,
        marker_in_payload: false,
        truncated_tail: true,
    },
];

pub const SMOKE_SCENARIOS: [ScenarioSpec; 3] = [
    SHARED_SCENARIOS[0],
    SHARED_SCENARIOS[4],
    SHARED_SCENARIOS[5],
];

pub fn scenarios_for_profile(profile: BenchmarkProfile) -> &'static [ScenarioSpec] {
    match profile {
        BenchmarkProfile::Smoke => &SMOKE_SCENARIOS,
        BenchmarkProfile::Full => &SHARED_SCENARIOS,
    }
}

pub fn payload_text(spec: ScenarioSpec) -> &'static str {
    if spec.marker_in_payload {
        "payload marker DLT\u{1} embedded"
    } else {
        "benchmark payload data"
    }
}

pub fn build_v1_verbose_payload(text: &str) -> Vec<u8> {
    let mut raw = Vec::new();
    let type_info = DLT_TYPE_INFO_STRG | DLT_SCOD_UTF8;
    raw.extend_from_slice(&type_info.to_be_bytes());
    raw.extend_from_slice(&((text.len() + 1) as u16).to_be_bytes());
    raw.extend_from_slice(text.as_bytes());
    raw.push(0);
    raw
}

pub fn build_v1_message(storage_ecu: [u8; 4], payload: &[u8], mcnt: u8) -> Vec<u8> {
    let mut msg = Vec::new();

    msg.extend_from_slice(b"DLT\x01");
    msg.extend_from_slice(&100u32.to_le_bytes());
    msg.extend_from_slice(&500u32.to_le_bytes());
    msg.extend_from_slice(&storage_ecu);

    let htyp: u8 = (1 << 5) | 0x01 | 0x02;
    msg.push(htyp);
    msg.push(mcnt);
    msg.extend_from_slice(&0u16.to_be_bytes());
    msg.push(0x01);
    msg.push(1);
    msg.extend_from_slice(b"BENC");
    msg.extend_from_slice(b"SCAN");
    msg.extend_from_slice(payload);

    let len = (msg.len() - 16) as u16;
    let len_off = 16 + 2;
    msg[len_off..len_off + 2].copy_from_slice(&len.to_be_bytes());
    msg
}

pub fn build_v2_message(storage_ecu: [u8; 4], payload_text: &str) -> Vec<u8> {
    V2MessageBuilder::new()
        .with_storage_ecu(&storage_ecu)
        .with_apid("BENC")
        .with_ctid("SCAN")
        .with_verbose_string(payload_text)
        .build()
}

pub fn build_v1_dataset(spec: ScenarioSpec) -> Vec<u8> {
    let payload = build_v1_verbose_payload(payload_text(spec));
    let mut data = Vec::new();

    for i in 0..spec.count {
        let storage_ecu = if spec.mixed_every > 0 && i % spec.mixed_every == 0 {
            *b"ECU2"
        } else {
            *b"ECU1"
        };
        data.extend_from_slice(&build_v1_message(storage_ecu, &payload, (i % 255) as u8));
    }

    if spec.truncated_tail && data.len() > 12 {
        data.truncate(data.len() - 12);
    }

    data
}

pub fn build_v2_dataset(spec: ScenarioSpec) -> Vec<u8> {
    let text = payload_text(spec);
    let mut data = Vec::new();

    for i in 0..spec.count {
        let storage_ecu = if spec.mixed_every > 0 && i % spec.mixed_every == 0 {
            *b"ECU2"
        } else {
            *b"ECU1"
        };
        data.extend_from_slice(&build_v2_message(storage_ecu, text));
    }

    if spec.truncated_tail && data.len() > 12 {
        data.truncate(data.len() - 12);
    }

    data
}
