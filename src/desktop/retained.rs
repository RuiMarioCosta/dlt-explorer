use crate::desktop::index::IndexLayer;
use crate::dlt;
use crate::dlt::error::ParseError;
use crate::dlt::payload::{MESSAGE_TYPE, decode_message_type_info};
use anyhow::{Result, anyhow};
use std::ops::Range;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LogTableRow {
    pub(crate) index: usize,
    pub(crate) timestamp: String,
    pub(crate) ecu: String,
    pub(crate) apid: String,
    pub(crate) ctid: String,
    pub(crate) kind: String,
    pub(crate) payload: String,
}

fn format_timestamp_ns(ns: u64) -> String {
    let seconds = ns / 1_000_000_000;
    let micros = (ns % 1_000_000_000) / 1_000;
    format!("{}.{:06}", seconds, micros)
}

pub(crate) fn format_message_type(mstp: u8, mtin: u8) -> String {
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
pub(crate) struct StructuredFilter {
    pub(crate) ecu_contains: String,
    pub(crate) apid_contains: String,
    pub(crate) ctid_contains: String,
    pub(crate) kind_contains: String,
}

impl StructuredFilter {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RenderedTextSearch {
    pub(crate) query: String,
    pub(crate) active_match_position: Option<usize>,
}

#[derive(Debug)]
pub(crate) enum RetainedDlt {
    V1(dlt::v1::Dlt),
    V2(dlt::v2::Dlt),
}

impl RetainedDlt {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.len(),
            Self::V2(dlt) => dlt.len(),
        }
    }

    pub(crate) fn unique_ecu_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_ecus().len(),
            Self::V2(dlt) => dlt.unique_ecus().len(),
        }
    }

    pub(crate) fn unique_apid_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_apids().len(),
            Self::V2(dlt) => dlt.unique_apids().len(),
        }
    }

    pub(crate) fn unique_ctid_count(&self) -> usize {
        match self {
            Self::V1(dlt) => dlt.unique_ctids().len(),
            Self::V2(dlt) => dlt.unique_ctids().len(),
        }
    }

    pub(crate) fn ecu(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.ecu(index),
            Self::V2(dlt) => dlt.ecu(index),
        }
    }

    pub(crate) fn apid(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.apid(index),
            Self::V2(dlt) => dlt.apid(index),
        }
    }

    pub(crate) fn ctid(&self, index: usize) -> &str {
        match self {
            Self::V1(dlt) => dlt.ctid(index),
            Self::V2(dlt) => dlt.ctid(index),
        }
    }

    pub(crate) fn message_type(&self, index: usize) -> u8 {
        match self {
            Self::V1(dlt) => dlt.message_type(index),
            Self::V2(dlt) => dlt.message_type(index),
        }
    }

    pub(crate) fn message_type_info(&self, index: usize) -> u8 {
        match self {
            Self::V1(dlt) => dlt.message_type_info(index),
            Self::V2(dlt) => dlt.message_type_info(index),
        }
    }

    pub(crate) fn row(&self, index: usize) -> LogTableRow {
        match self {
            Self::V1(dlt) => LogTableRow {
                index,
                timestamp: format_timestamp_ns(dlt.storage_timestamp_ns(index)),
                ecu: display_field(dlt.ecu(index)),
                apid: display_field(dlt.apid(index)),
                ctid: display_field(dlt.ctid(index)),
                kind: format_message_type(dlt.message_type(index), dlt.message_type_info(index)),
                payload: display_payload(dlt.payload_text(index)),
            },
            Self::V2(dlt) => LogTableRow {
                index,
                timestamp: format_timestamp_ns(dlt.storage_timestamp_ns(index)),
                ecu: display_field(dlt.ecu(index)),
                apid: display_field(dlt.apid(index)),
                ctid: display_field(dlt.ctid(index)),
                kind: format_message_type(dlt.message_type(index), dlt.message_type_info(index)),
                payload: display_payload(dlt.payload_text(index)),
            },
        }
    }

    pub(crate) fn rendered_row_text(&self, index: usize) -> String {
        let row = self.row(index);
        format!(
            "{} {} {} {} {} {} {}",
            row.index, row.timestamp, row.ecu, row.apid, row.ctid, row.kind, row.payload
        )
    }
}

#[derive(Debug)]
pub(crate) struct RetainedDataSet {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) version: u8,
    pub(crate) parse_errors: Vec<ParseError>,
    dlt: RetainedDlt,
    index: IndexLayer,
    pub(crate) active_filter: StructuredFilter,
    rendered_search: RenderedTextSearch,
    selected_visible_row: Option<usize>,
    pending_scroll_to_selected: bool,
}

impl RetainedDataSet {
    pub(crate) fn message_count(&self) -> usize {
        self.dlt.len()
    }

    pub(crate) fn file_count(&self) -> usize {
        self.paths.len()
    }

    pub(crate) fn parse_error_count(&self) -> usize {
        self.parse_errors.len()
    }

    pub(crate) fn unique_ecu_count(&self) -> usize {
        self.dlt.unique_ecu_count()
    }

    pub(crate) fn unique_apid_count(&self) -> usize {
        self.dlt.unique_apid_count()
    }

    pub(crate) fn unique_ctid_count(&self) -> usize {
        self.dlt.unique_ctid_count()
    }

    pub(crate) fn visible_message_count(&self) -> usize {
        self.index.visible_count()
    }

    pub(crate) fn rebuild_index(&mut self) {
        let previous_selected_index = self.selected_row_index();
        self.index = IndexLayer::from_filter_and_search(
            &self.dlt,
            &self.active_filter,
            self.rendered_search.query.as_str(),
        );
        self.rebuild_rendered_search(previous_selected_index);
    }

    pub(crate) fn set_structured_filter(&mut self, filter: StructuredFilter) {
        self.active_filter = filter;
        self.rebuild_index();
    }

    pub(crate) fn clear_filter(&mut self) {
        self.active_filter.reset();
        self.rebuild_index();
    }

    pub(crate) fn visible_rows(&self, range: Range<usize>) -> Vec<LogTableRow> {
        self.index.visible_rows(&self.dlt, range)
    }

    pub(crate) fn set_rendered_search_query(&mut self, query: String) {
        self.rendered_search.query = query;
        self.rebuild_index();
    }

    pub(crate) fn rendered_search_query(&self) -> &str {
        self.rendered_search.query.as_str()
    }

    pub(crate) fn rendered_search_match_count(&self) -> usize {
        self.index.rendered_search_match_count()
    }

    pub(crate) fn rendered_search_active_ordinal(&self) -> Option<usize> {
        let active_position = self.rendered_search.active_match_position?;
        self.index.rendered_search_match_ordinal(active_position)
    }

    pub(crate) fn select_next_rendered_match(&mut self) -> bool {
        let total = self.index.rendered_search_match_count();
        if total == 0 {
            return false;
        }

        let current_idx = self
            .rendered_search
            .active_match_position
            .and_then(|active| self.index.rendered_search_match_ordinal(active))
            .map(|ordinal| ordinal - 1)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % total;
        self.apply_active_match_by_index(next_idx)
    }

    pub(crate) fn select_previous_rendered_match(&mut self) -> bool {
        let total = self.index.rendered_search_match_count();
        if total == 0 {
            return false;
        }

        let current_idx = self
            .rendered_search
            .active_match_position
            .and_then(|active| self.index.rendered_search_match_ordinal(active))
            .map(|ordinal| ordinal - 1)
            .unwrap_or(0);
        let prev_idx = (current_idx + total - 1) % total;
        self.apply_active_match_by_index(prev_idx)
    }

    pub(crate) fn selected_row_index(&self) -> Option<usize> {
        let selected_pos = self.selected_visible_row?;
        self.index.visible_index_at(selected_pos)
    }

    pub(crate) fn selected_visible_row(&self) -> Option<usize> {
        self.selected_visible_row
    }

    pub(crate) fn take_pending_scroll_to_selected(&mut self) -> bool {
        let pending = self.pending_scroll_to_selected;
        self.pending_scroll_to_selected = false;
        pending
    }

    pub(crate) fn select_visible_row(&mut self, position: usize, request_scroll: bool) {
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

        if self.index.is_rendered_search_match_position(position) {
            self.rendered_search.active_match_position = Some(position);
        }
    }

    #[cfg(test)]
    pub(crate) fn rendered_row_text_for_index(&self, index: usize) -> String {
        self.dlt.rendered_row_text(index)
    }

    fn rebuild_rendered_search(&mut self, previous_selected_index: Option<usize>) {
        if self.rendered_search.query.is_empty() {
            self.rendered_search.active_match_position = None;

            self.selected_visible_row = previous_selected_index
                .and_then(|selected| self.index.position_for_index(selected));
            return;
        }

        if self.index.rendered_search_match_count() == 0 {
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
                    .filter(|&pos| self.index.is_rendered_search_match_position(pos))
            })
            .unwrap_or(0);

        self.selected_visible_row = Some(preferred);
        self.rendered_search.active_match_position = Some(preferred);
    }

    fn apply_active_match_by_index(&mut self, match_index: usize) -> bool {
        let Some(visible_position) = self.index.rendered_search_match_position(match_index) else {
            return false;
        };

        self.rendered_search.active_match_position = Some(visible_position);
        self.selected_visible_row = Some(visible_position);
        self.pending_scroll_to_selected = true;
        true
    }
}

pub(crate) fn load_retained_dataset(paths: Vec<PathBuf>) -> Result<RetainedDataSet> {
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
            index: IndexLayer::empty(),
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
            index: IndexLayer::empty(),
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