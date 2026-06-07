use crate::dlt;
use crate::dlt::error::ParseError;
use crate::dlt::payload::{MESSAGE_TYPE, decode_message_type_info};
use anyhow::{Result, anyhow};
use eframe::egui;
use std::ops::Range;
use std::path::PathBuf;

const TABLE_COL_TIMESTAMP: f32 = 140.0;
const TABLE_COL_ECU: f32 = 70.0;
const TABLE_COL_APID: f32 = 70.0;
const TABLE_COL_CTID: f32 = 70.0;
const TABLE_COL_TYPE: f32 = 140.0;
const TABLE_ROW_HEIGHT: f32 = 20.0;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogTableRow {
    index: usize,
    timestamp: String,
    ecu: String,
    apid: String,
    ctid: String,
    kind: String,
    payload: String,
}

fn format_timestamp_ns(ns: u64) -> String {
    let seconds = ns / 1_000_000_000;
    let micros = (ns % 1_000_000_000) / 1_000;
    format!("{}.{:06}", seconds, micros)
}

fn format_message_type(mstp: u8, mtin: u8) -> String {
    let mstp_usize = mstp as usize;
    let family = MESSAGE_TYPE.get(mstp_usize).copied().unwrap_or("");
    let info = decode_message_type_info(mstp_usize, mtin as usize);

    if family.is_empty() {
        return format!("unknown({mstp},{mtin})");
    }
    if info.is_empty() {
        return family.to_string();
    }

    format!("{family}/{info}")
}

fn display_field(value: &str) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value.to_string()
    }
}

fn display_payload(value: String) -> String {
    if value.is_empty() {
        "-".to_string()
    } else {
        value
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StructuredFilter {
    ecu_contains: String,
    apid_contains: String,
    ctid_contains: String,
    kind_contains: String,
}

impl StructuredFilter {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexLayer {
    visible_indices: Vec<usize>,
}

impl IndexLayer {
    fn from_filter(dlt: &RetainedDlt, filter: &StructuredFilter) -> Self {
        let visible_indices = (0..dlt.len())
            .filter(|&index| {
                let ecu_matches = contains_ignore_case(
                    dlt.ecu(index),
                    filter.ecu_contains.as_str(),
                );
                let apid_matches = contains_ignore_case(
                    dlt.apid(index),
                    filter.apid_contains.as_str(),
                );
                let ctid_matches = contains_ignore_case(
                    dlt.ctid(index),
                    filter.ctid_contains.as_str(),
                );
                let kind = format_message_type(
                    dlt.message_type(index),
                    dlt.message_type_info(index),
                );
                let kind_matches = contains_ignore_case(
                    kind.as_str(),
                    filter.kind_contains.as_str(),
                );

                ecu_matches && apid_matches && ctid_matches && kind_matches
            })
            .collect();

        Self { visible_indices }
    }

    fn visible_count(&self) -> usize {
        self.visible_indices.len()
    }

    fn visible_rows(
        &self,
        dlt: &RetainedDlt,
        range: Range<usize>,
    ) -> Vec<LogTableRow> {
        let total_rows = self.visible_count();
        let start = range.start.min(total_rows);
        let end = range.end.min(total_rows);

        self.visible_indices[start..end]
            .iter()
            .copied()
            .map(|idx| dlt.row(idx))
            .collect()
    }
}

fn contains_ignore_case(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    value.to_ascii_lowercase().contains(&query.to_ascii_lowercase())
}

#[derive(Debug)]
enum RetainedDlt {
    V1(dlt::v1::Dlt),
    V2(dlt::v2::Dlt),
}

impl RetainedDlt {
    fn len(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.len(),
            Self::V2(dlt) => dlt.len(),
        }
    }

    fn unique_ecu_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_ecus().len(),
            Self::V2(dlt) => dlt.unique_ecus().len(),
        }
    }

    fn unique_apid_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_apids().len(),
            Self::V2(dlt) => dlt.unique_apids().len(),
        }
    }

    fn unique_ctid_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_ctids().len(),
            Self::V2(dlt) => dlt.unique_ctids().len(),
        }
    }

    fn ecu(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.ecu(index),
            Self::V2(dlt) => dlt.ecu(index),
        }
    }

    fn apid(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.apid(index),
            Self::V2(dlt) => dlt.apid(index),
        }
    }

    fn ctid(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.ctid(index),
            Self::V2(dlt) => dlt.ctid(index),
        }
    }

    fn message_type(&self, index: usize) -> u8 {
        match self {
            Self::V1(dlt) => dlt.message_type(index),
            Self::V2(dlt) => dlt.message_type(index),
        }
    }

    fn message_type_info(&self, index: usize) -> u8 {
        match self {
            Self::V1(dlt) => dlt.message_type_info(index),
            Self::V2(dlt) => dlt.message_type_info(index),
        }
    }

    fn row(&self, index: usize) -> LogTableRow {
        match self {
            Self::V1(dlt) => LogTableRow {
                index,
                timestamp: format_timestamp_ns(dlt.storage_timestamp_ns(index)),
                ecu: display_field(dlt.ecu(index)),
                apid: display_field(dlt.apid(index)),
                ctid: display_field(dlt.ctid(index)),
                kind: format_message_type(
                    dlt.message_type(index),
                    dlt.message_type_info(index),
                ),
                payload: display_payload(dlt.payload_text(index)),
            },
            Self::V2(dlt) => LogTableRow {
                index,
                timestamp: format_timestamp_ns(dlt.storage_timestamp_ns(index)),
                ecu: display_field(dlt.ecu(index)),
                apid: display_field(dlt.apid(index)),
                ctid: display_field(dlt.ctid(index)),
                kind: format_message_type(
                    dlt.message_type(index),
                    dlt.message_type_info(index),
                ),
                payload: display_payload(dlt.payload_text(index)),
            },
        }
    }
}

#[derive(Debug)]
struct RetainedDataSet {
    paths: Vec<PathBuf>,
    version: u8,
    parse_errors: Vec<ParseError>,
    dlt: RetainedDlt,
    index: IndexLayer,
    active_filter: StructuredFilter,
}

impl RetainedDataSet {
    fn message_count(&self) -> usize {
        self.dlt.len()
    }

    fn file_count(&self) -> usize {
        self.paths.len()
    }

    fn parse_error_count(&self) -> usize {
        self.parse_errors.len()
    }

    fn unique_ecu_count(&self) -> usize {
        self.dlt.unique_ecu_count()
    }

    fn unique_apid_count(&self) -> usize {
        self.dlt.unique_apid_count()
    }

    fn unique_ctid_count(&self) -> usize {
        self.dlt.unique_ctid_count()
    }

    fn visible_message_count(&self) -> usize {
        self.index.visible_count()
    }

    fn rebuild_index(&mut self) {
        self.index = IndexLayer::from_filter(&self.dlt, &self.active_filter);
    }

    fn clear_filter(&mut self) {
        self.active_filter.reset();
        self.rebuild_index();
    }

    fn visible_rows(&self, range: Range<usize>) -> Vec<LogTableRow> {
        self.index.visible_rows(&self.dlt, range)
    }
}

fn render_structured_filter_controls(ui: &mut egui::Ui, data: &mut RetainedDataSet) {
    ui.separator();
    ui.label("Structured Filter");

    let mut changed = false;
    let mut clear_clicked = false;
    ui.horizontal(|ui| {
        let filter = &mut data.active_filter;
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut filter.ecu_contains)
                    .hint_text("ECU contains"),
            )
            .changed();
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut filter.apid_contains)
                    .hint_text("APID contains"),
            )
            .changed();
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut filter.ctid_contains)
                    .hint_text("CTID contains"),
            )
            .changed();
        changed |= ui
            .add(
                egui::TextEdit::singleline(&mut filter.kind_contains)
                    .hint_text("Type contains"),
            )
            .changed();

        clear_clicked = ui.button("Clear").clicked();
    });

    if clear_clicked {
        data.clear_filter();
    } else if changed {
        data.rebuild_index();
    }

    ui.label(format!(
        "Visible rows: {} / {}",
        data.visible_message_count(),
        data.message_count()
    ));
}

fn render_log_table(ui: &mut egui::Ui, data: &RetainedDataSet) {
    ui.separator();
    ui.label("Log Table");

    ui.horizontal(|ui| {
        ui.add_sized(
            [50.0, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("#").strong()),
        );
        ui.add_sized(
            [TABLE_COL_TIMESTAMP, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("Timestamp").strong()),
        );
        ui.add_sized(
            [TABLE_COL_ECU, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("ECU").strong()),
        );
        ui.add_sized(
            [TABLE_COL_APID, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("APID").strong()),
        );
        ui.add_sized(
            [TABLE_COL_CTID, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("CTID").strong()),
        );
        ui.add_sized(
            [TABLE_COL_TYPE, TABLE_ROW_HEIGHT],
            egui::Label::new(egui::RichText::new("Type").strong()),
        );
        ui.label(egui::RichText::new("Payload").strong());
    });
    ui.separator();

    let total_rows = data.visible_message_count();
    if total_rows == 0 {
        ui.label("No log rows match the structured filter.");
        return;
    }

    egui::ScrollArea::vertical()
        .id_salt("desktop_log_table")
        .auto_shrink([false, false])
        .show_rows(ui, TABLE_ROW_HEIGHT, total_rows, |ui, row_range| {
            for row in data.visible_rows(row_range) {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [50.0, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.index.to_string()),
                    );
                    ui.add_sized(
                        [TABLE_COL_TIMESTAMP, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.timestamp),
                    );
                    ui.add_sized(
                        [TABLE_COL_ECU, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.ecu),
                    );
                    ui.add_sized(
                        [TABLE_COL_APID, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.apid),
                    );
                    ui.add_sized(
                        [TABLE_COL_CTID, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.ctid),
                    );
                    ui.add_sized(
                        [TABLE_COL_TYPE, TABLE_ROW_HEIGHT],
                        egui::Label::new(row.kind),
                    );
                    ui.label(row.payload);
                });
            }
        });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DesktopAppState {
    Idle,
    Loading,
    Loaded,
    Error(String),
}

#[derive(Debug)]
pub(crate) struct DesktopModel {
    state: DesktopAppState,
    retained: Option<RetainedDataSet>,
}

impl Default for DesktopModel {
    fn default() -> Self {
        Self {
            state: DesktopAppState::Idle,
            retained: None,
        }
    }
}

impl DesktopModel {
    pub(crate) fn state(&self) -> &DesktopAppState {
        &self.state
    }

    pub(crate) fn begin_loading(&mut self) {
        self.state = DesktopAppState::Loading;
    }

    fn loaded_data_mut(&mut self) -> Option<&mut RetainedDataSet> {
        self.retained.as_mut()
    }

    fn loading_succeeded(&mut self, data: RetainedDataSet) {
        self.retained = Some(data);
        self.state = DesktopAppState::Loaded;
    }

    pub(crate) fn loading_failed(&mut self, message: impl Into<String>) {
        self.retained = None;
        self.state = DesktopAppState::Error(message.into());
    }

    pub(crate) fn reset_idle(&mut self) {
        self.retained = None;
        self.state = DesktopAppState::Idle;
    }
}

fn load_retained_dataset(paths: Vec<PathBuf>) -> Result<RetainedDataSet> {
    if paths.is_empty() {
        return Err(anyhow!("No DLT paths selected"));
    }

    let version = dlt::detect_version(&paths[0])?;
    for path in &paths[1..] {
        let candidate = dlt::detect_version(path)?;
        if candidate != version {
            return Err(anyhow!(
                "Mixed DLT versions: first file is v{} but {:?} is v{}",
                version,
                path,
                candidate
            ));
        }
    }

    if version == 1 {
        let (dlt, parse_errors) = dlt::v1::Dlt::open(paths.clone())?;
        let mut data = RetainedDataSet {
            paths,
            version,
            parse_errors,
            dlt: RetainedDlt::V1(dlt),
            index: IndexLayer {
                visible_indices: Vec::new(),
            },
            active_filter: StructuredFilter::default(),
        };
        data.rebuild_index();
        Ok(data)
    } else if version == 2 {
        let (dlt, parse_errors) = dlt::v2::Dlt::open(paths.clone())?;
        let mut data = RetainedDataSet {
            paths,
            version,
            parse_errors,
            dlt: RetainedDlt::V2(dlt),
            index: IndexLayer {
                visible_indices: Vec::new(),
            },
            active_filter: StructuredFilter::default(),
        };
        data.rebuild_index();
        Ok(data)
    } else {
        Err(anyhow!("Unsupported DLT version: {}", version))
    }
}

#[derive(Default)]
struct DesktopShell {
    model: DesktopModel,
}

impl eframe::App for DesktopShell {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open DLT files").clicked() {
                    self.model.begin_loading();

                    let Some(paths) = rfd::FileDialog::new()
                        .add_filter("DLT files", &["dlt"])
                        .pick_files()
                    else {
                        self.model.reset_idle();
                        return;
                    };

                    match load_retained_dataset(paths) {
                        Ok(data) => self.model.loading_succeeded(data),
                        Err(err) => self.model.loading_failed(err.to_string()),
                    }
                }

                if ui.button("Reset").clicked() {
                    self.model.reset_idle();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DLT Explorer");
            ui.separator();

            match self.model.state().clone() {
                DesktopAppState::Idle => {
                    ui.label("State: idle");
                    ui.label("No DLT files loaded.");
                }
                DesktopAppState::Loading => {
                    ui.label("State: loading");
                    ui.spinner();
                    ui.label("Loading DLT data...");
                }
                DesktopAppState::Loaded => {
                    ui.label("State: loaded");

                    if let Some(data) = self.model.loaded_data_mut() {
                        ui.label(format!(
                            "Loaded {} message(s) from {} file(s) (DLT v{}).",
                            data.message_count(),
                            data.file_count(),
                            data.version
                        ));
                        ui.label(format!(
                            "Metadata: {} ECU(s), {} APID(s), {} CTID(s).",
                            data.unique_ecu_count(),
                            data.unique_apid_count(),
                            data.unique_ctid_count()
                        ));

                        let parse_error_count = data.parse_error_count();
                        if parse_error_count > 0 {
                            ui.colored_label(
                                egui::Color32::from_rgb(180, 80, 0),
                                format!("Parse warnings: {}", parse_error_count),
                            );
                            ui.separator();

                            for (idx, err) in data.parse_errors.iter().take(10).enumerate() {
                                ui.label(format!("{}. {}", idx + 1, err));
                            }

                            if parse_error_count > 10 {
                                ui.label(format!(
                                    "... and {} more parse warning(s)",
                                    parse_error_count - 10
                                ));
                            }
                        }

                        render_structured_filter_controls(ui, data);
                        render_log_table(ui, data);
                    }
                }
                DesktopAppState::Error(message) => {
                    ui.colored_label(egui::Color32::RED, "State: error");
                    ui.label(message);
                }
            }
        });
    }
}

pub(crate) fn run_desktop_shell() -> Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Dlt Explorer",
        options,
        Box::new(|_cc| Ok(Box::new(DesktopShell::default()))),
    )
    .map_err(|e| anyhow!(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopAppState, DesktopModel, StructuredFilter, format_message_type,
        load_retained_dataset,
    };
    use std::path::PathBuf;

    #[test]
    fn desktop_model_starts_idle() {
        let model = DesktopModel::default();
        assert_eq!(model.state(), &DesktopAppState::Idle);
    }

    #[test]
    fn desktop_model_transitions_loading_and_error() {
        let mut model = DesktopModel::default();

        model.begin_loading();
        assert_eq!(model.state(), &DesktopAppState::Loading);

        model.loading_failed("boom");
        assert_eq!(model.state(), &DesktopAppState::Error("boom".to_string()));

        model.reset_idle();
        assert_eq!(model.state(), &DesktopAppState::Idle);
    }

    #[test]
    fn retained_loader_rejects_empty_path_list() {
        let result = load_retained_dataset(Vec::new());
        assert!(result.is_err());
    }

    #[test]
    fn retained_loader_loads_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let result = load_retained_dataset(vec![path]);
        assert!(result.is_ok());
    }

    #[test]
    fn retained_visible_rows_are_slice_based() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let data = load_retained_dataset(vec![path]).expect("fixture should load");
        let total = data.message_count();
        assert!(total >= 2);

        let rows = data.visible_rows(1..2);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].index, 1);
        assert!(!rows[0].timestamp.is_empty());
    }

    #[test]
    fn retained_visible_rows_clamp_out_of_bounds_ranges() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let data = load_retained_dataset(vec![path]).expect("fixture should load");
        let total = data.message_count();
        let rows = data.visible_rows(total..(total + 50));
        assert!(rows.is_empty());
    }

    #[test]
    fn structured_filter_returns_no_rows_when_no_match() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        data.active_filter = StructuredFilter {
            ecu_contains: "definitely-not-a-real-ecu".to_string(),
            ..StructuredFilter::default()
        };
        data.rebuild_index();

        assert_eq!(data.visible_message_count(), 0);
        assert!(data.visible_rows(0..10).is_empty());
    }

    #[test]
    fn structured_filter_updates_visible_results_when_changed() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        let total = data.message_count();
        assert!(total > 0);

        data.active_filter = StructuredFilter {
            kind_contains: "control".to_string(),
            ..StructuredFilter::default()
        };
        data.rebuild_index();
        let control_count = data.visible_message_count();
        assert!(control_count > 0);
        assert!(control_count <= total);

        data.active_filter.kind_contains = "no-such-message-type".to_string();
        data.rebuild_index();
        assert_eq!(data.visible_message_count(), 0);

        data.clear_filter();
        assert_eq!(data.visible_message_count(), total);
    }

    #[test]
    fn message_type_format_uses_family_and_info_when_present() {
        assert_eq!(format_message_type(0, 4), "log/info");
        assert_eq!(format_message_type(3, 2), "control/response");
        assert_eq!(format_message_type(3, 9), "control");
    }
}
