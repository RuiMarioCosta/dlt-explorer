use crate::desktop::retained::{RetainedDataSet, load_retained_dataset};
use anyhow::Result;
use std::path::PathBuf;

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

    pub(crate) fn loaded_data_mut(&mut self) -> Option<&mut RetainedDataSet> {
        self.retained.as_mut()
    }

    pub(crate) fn loading_succeeded(&mut self, data: RetainedDataSet) {
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