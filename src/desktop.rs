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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct RenderedTextSearch {
    query: String,
    match_positions: Vec<usize>,
    active_match_position: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexLayer {
    visible_indices: Vec<usize>,
}

impl IndexLayer {
    fn from_filter_and_search(
        dlt: &RetainedDlt,
        filter: &StructuredFilter,
        rendered_search_query: &str,
    ) -> Self {
        let structured_filtered_indices: Vec<usize> = (0..dlt.len())
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

        if rendered_search_query.is_empty() {
            return Self {
                visible_indices: structured_filtered_indices,
            };
        }

        let visible_indices = structured_filtered_indices
            .into_iter()
            .filter(|&index| {
                let rendered = dlt.rendered_row_text(index);
                contains_ignore_case(rendered.as_str(), rendered_search_query)
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

    fn visible_index_at(&self, position: usize) -> Option<usize> {
        self.visible_indices.get(position).copied()
    }

    fn position_for_index(&self, index: usize) -> Option<usize> {
        self.visible_indices.iter().position(|&value| value == index)
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

    fn rendered_row_text(&self, index: usize) -> String {
        let row = self.row(index);
        format!(
            "{} {} {} {} {} {} {}",
            row.index,
            row.timestamp,
            row.ecu,
            row.apid,
            row.ctid,
            row.kind,
            row.payload
        )
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
    rendered_search: RenderedTextSearch,
    selected_visible_row: Option<usize>,
    pending_scroll_to_selected: bool,
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
        let previous_selected_index = self.selected_row_index();
        self.index = IndexLayer::from_filter_and_search(
            &self.dlt,
            &self.active_filter,
            self.rendered_search.query.as_str(),
        );
        self.rebuild_rendered_search(previous_selected_index);
    }

    fn clear_filter(&mut self) {
        self.active_filter.reset();
        self.rebuild_index();
    }

    fn visible_rows(&self, range: Range<usize>) -> Vec<LogTableRow> {
        self.index.visible_rows(&self.dlt, range)
    }

    fn set_rendered_search_query(&mut self, query: String) {
        self.rendered_search.query = query;
        self.rebuild_index();
    }

    fn rendered_search_query_mut(&mut self) -> &mut String {
        &mut self.rendered_search.query
    }

    fn rendered_search_match_count(&self) -> usize {
        self.rendered_search.match_positions.len()
    }

    fn rendered_search_active_ordinal(&self) -> Option<usize> {
        let active_position = self.rendered_search.active_match_position?;
        self.rendered_search
            .match_positions
            .iter()
            .position(|&pos| pos == active_position)
            .map(|idx| idx + 1)
    }

    fn select_next_rendered_match(&mut self) -> bool {
        let total = self.rendered_search.match_positions.len();
        if total == 0 {
            return false;
        }

        let current_idx = self
            .rendered_search
            .active_match_position
            .and_then(|active| {
                self.rendered_search
                    .match_positions
                    .iter()
                    .position(|&pos| pos == active)
            })
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % total;
        self.apply_active_match_by_index(next_idx)
    }

    fn select_previous_rendered_match(&mut self) -> bool {
        let total = self.rendered_search.match_positions.len();
        if total == 0 {
            return false;
        }

        let current_idx = self
            .rendered_search
            .active_match_position
            .and_then(|active| {
                self.rendered_search
                    .match_positions
                    .iter()
                    .position(|&pos| pos == active)
            })
            .unwrap_or(0);
        let prev_idx = (current_idx + total - 1) % total;
        self.apply_active_match_by_index(prev_idx)
    }

    fn selected_row_index(&self) -> Option<usize> {
        let selected_pos = self.selected_visible_row?;
        self.index.visible_index_at(selected_pos)
    }

    fn selected_visible_row(&self) -> Option<usize> {
        self.selected_visible_row
    }

    fn take_pending_scroll_to_selected(&mut self) -> bool {
        let pending = self.pending_scroll_to_selected;
        self.pending_scroll_to_selected = false;
        pending
    }

    fn select_visible_row(&mut self, position: usize, request_scroll: bool) {
        if self.index.visible_index_at(position).is_none() {
            return;
        }

        self.selected_visible_row = Some(position);
        if request_scroll {
            self.pending_scroll_to_selected = true;
        }
        if self.rendered_search.query.is_empty() {
            self.rendered_search.active_match_position = None;
            return;
        }

        if self
            .rendered_search
            .match_positions
            .contains(&position)
        {
            self.rendered_search.active_match_position = Some(position);
        }
    }

    fn rebuild_rendered_search(&mut self, previous_selected_index: Option<usize>) {
        if self.rendered_search.query.is_empty() {
            self.rendered_search.match_positions.clear();
            self.rendered_search.active_match_position = None;

            self.selected_visible_row = previous_selected_index
                .and_then(|selected| self.index.position_for_index(selected));
            return;
        }

        self.rendered_search.match_positions =
            (0..self.index.visible_count()).collect();
        if self.rendered_search.match_positions.is_empty() {
            self.rendered_search.active_match_position = None;
            self.selected_visible_row = None;
            return;
        }

        let preferred = self
            .index
            .position_for_index(previous_selected_index.unwrap_or(usize::MAX))
            .or_else(|| {
                self.rendered_search
                    .active_match_position
                    .filter(|&pos| pos < self.rendered_search.match_positions.len())
            })
            .unwrap_or(0);

        self.selected_visible_row = Some(preferred);
        self.rendered_search.active_match_position = Some(preferred);
    }

    fn apply_active_match_by_index(&mut self, match_index: usize) -> bool {
        let Some(&visible_position) =
            self.rendered_search.match_positions.get(match_index)
        else {
            return false;
        };

        self.rendered_search.active_match_position = Some(visible_position);
        self.selected_visible_row = Some(visible_position);
        self.pending_scroll_to_selected = true;
        true
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

fn render_rendered_search_controls(ui: &mut egui::Ui, data: &mut RetainedDataSet) {
    ui.separator();
    ui.label("Rendered Text Search");

    let mut query_changed = false;
    let mut clear_clicked = false;
    let mut prev_clicked = false;
    let mut next_clicked = false;

    ui.horizontal(|ui| {
        query_changed = ui
            .add(
                egui::TextEdit::singleline(data.rendered_search_query_mut())
                    .hint_text("Search rendered message text"),
            )
            .changed();

        clear_clicked = ui.button("Clear").clicked();
        prev_clicked = ui.button("Prev").clicked();
        next_clicked = ui.button("Next").clicked();
    });

    if query_changed {
        let query = data.rendered_search.query.clone();
        data.set_rendered_search_query(query);
    }

    if clear_clicked {
        data.set_rendered_search_query(String::new());
    }

    if prev_clicked {
        data.select_previous_rendered_match();
    }
    if next_clicked {
        data.select_next_rendered_match();
    }

    let match_count = data.rendered_search_match_count();
    if match_count == 0 {
        ui.label("Matches: 0");
    } else {
        let active = data.rendered_search_active_ordinal().unwrap_or(1);
        ui.label(format!("Matches: {} (active {}/{})", match_count, active, match_count));
    }

    if let Some(selected) = data.selected_row_index() {
        ui.label(format!("Selected row: {}", selected));
    }
}

fn render_log_table_with_navigation(ui: &mut egui::Ui, data: &mut RetainedDataSet) {
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
        ui.label("No log rows match the active query.");
        return;
    }

    let selected_visible_row = data.selected_visible_row();
    let should_scroll_to_selection = data.take_pending_scroll_to_selected();

    egui::ScrollArea::vertical()
        .id_salt("desktop_log_table")
        .auto_shrink([false, false])
        .show_rows(ui, TABLE_ROW_HEIGHT, total_rows, |ui, row_range| {
            for (offset, row) in data.visible_rows(row_range.clone()).into_iter().enumerate() {
                let visible_position = row_range.start + offset;
                let is_selected = selected_visible_row == Some(visible_position);

                let response = ui.horizontal(|ui| {
                    ui.add_sized(
                        [50.0, TABLE_ROW_HEIGHT],
                        egui::Label::new(
                            egui::RichText::new(row.index.to_string())
                                .strong()
                                .background_color(if is_selected {
                                    egui::Color32::from_rgb(34, 74, 125)
                                } else {
                                    egui::Color32::TRANSPARENT
                                }),
                        ),
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

                if is_selected && should_scroll_to_selection {
                    ui.scroll_to_rect(response.response.rect, Some(egui::Align::Center));
                }

                if response.response.clicked() {
                    data.select_visible_row(visible_position, false);
                }
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
            rendered_search: RenderedTextSearch::default(),
            selected_visible_row: None,
            pending_scroll_to_selected: false,
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
            rendered_search: RenderedTextSearch::default(),
            selected_visible_row: None,
            pending_scroll_to_selected: false,
        };
        data.rebuild_index();
        Ok(data)
    } else {
        Err(anyhow!("Unsupported DLT version: {}", version))
    }
}

#[derive(Debug)]
#[doc(hidden)]
pub struct DesktopBenchmarkHarness {
    data: RetainedDataSet,
}

impl DesktopBenchmarkHarness {
    pub fn load(paths: Vec<PathBuf>) -> Result<Self> {
        let data = load_retained_dataset(paths)?;
        Ok(Self { data })
    }

    pub fn set_kind_filter_contains(&mut self, query: &str) {
        self.data.active_filter.kind_contains = query.to_string();
        self.data.rebuild_index();
    }

    pub fn set_rendered_search_query(&mut self, query: &str) {
        self.data.set_rendered_search_query(query.to_string());
    }

    pub fn clear_rendered_search_query(&mut self) {
        self.data.set_rendered_search_query(String::new());
    }

    pub fn visible_message_count(&self) -> usize {
        self.data.visible_message_count()
    }

    pub fn first_visible_timestamp(&self) -> Option<String> {
        self.data
            .visible_rows(0..1)
            .into_iter()
            .next()
            .map(|row| row.timestamp)
    }

    pub fn read_visible_row_window(&self, start: usize, len: usize) -> usize {
        let end = start.saturating_add(len);
        self.data.visible_rows(start..end).len()
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
                        render_rendered_search_controls(ui, data);
                        render_log_table_with_navigation(ui, data);
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

    #[test]
    fn rendered_text_search_matches_user_visible_row_text() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_number_and_text.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        let first_row = data
            .visible_rows(0..1)
            .into_iter()
            .next()
            .expect("fixture should contain one row");
        data.set_rendered_search_query(first_row.timestamp.clone());

        assert!(data.rendered_search_match_count() > 0);
        assert!(data.selected_row_index().is_some());
    }

    #[test]
    fn rendered_text_search_navigation_updates_selected_row() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        data.set_rendered_search_query("control".to_string());

        let total_matches = data.rendered_search_match_count();
        assert!(total_matches > 0);
        let first_selected = data.selected_row_index();

        assert!(data.select_next_rendered_match());
        let next_selected = data.selected_row_index();
        assert!(next_selected.is_some());
        if total_matches > 1 {
            assert_ne!(first_selected, next_selected);
        }

        assert!(data.select_previous_rendered_match());
        assert!(data.selected_row_index().is_some());
    }

    #[test]
    fn combined_query_pipeline_restricts_visible_rows() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        data.active_filter = StructuredFilter {
            kind_contains: "control".to_string(),
            ..StructuredFilter::default()
        };
        data.rebuild_index();
        let filtered_count = data.visible_message_count();
        assert!(filtered_count > 0);

        let marker = data
            .visible_rows(0..1)
            .into_iter()
            .next()
            .expect("at least one filtered row")
            .timestamp;

        data.set_rendered_search_query(marker.clone());
        let combined_count = data.visible_message_count();
        assert!(combined_count > 0);
        assert!(combined_count <= filtered_count);

        for row in data.visible_rows(0..combined_count) {
            let rendered = data.dlt.rendered_row_text(row.index);
            assert!(rendered.to_ascii_lowercase().contains(&marker.to_ascii_lowercase()));
        }
    }

    #[test]
    fn combined_query_handles_empty_results_and_transitions() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        data.active_filter = StructuredFilter {
            kind_contains: "control".to_string(),
            ..StructuredFilter::default()
        };
        data.rebuild_index();
        let filtered_count = data.visible_message_count();
        assert!(filtered_count > 0);

        data.set_rendered_search_query("no-such-rendered-text-token".to_string());
        assert_eq!(data.visible_message_count(), 0);
        assert_eq!(data.rendered_search_match_count(), 0);
        assert!(data.selected_row_index().is_none());

        data.set_rendered_search_query(String::new());
        assert_eq!(data.visible_message_count(), filtered_count);
    }

    #[test]
    fn combined_query_preserves_selection_when_row_stays_visible() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut data = load_retained_dataset(vec![path]).expect("fixture should load");
        data.active_filter = StructuredFilter {
            kind_contains: "control".to_string(),
            ..StructuredFilter::default()
        };
        data.rebuild_index();

        let selected_position = if data.visible_message_count() > 1 { 1 } else { 0 };
        data.select_visible_row(selected_position, false);
        let selected_index = data.selected_row_index().expect("selection should exist");
        let selected_timestamp = data.dlt.row(selected_index).timestamp;

        data.set_rendered_search_query(selected_timestamp);

        assert_eq!(data.selected_row_index(), Some(selected_index));
        assert!(data.visible_message_count() > 0);
    }
}
