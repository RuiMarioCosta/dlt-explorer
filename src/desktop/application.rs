use crate::desktop::retained::{RetainedDataSet, StructuredFilter, load_retained_dataset};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum DesktopIntent {
    OpenFilesRequested,
    OpenFilesCancelled,
    LoadSucceeded(RetainedDataSet),
    LoadFailed(String),
    ResetRequested,
    StructuredFilterUpdated(StructuredFilter),
    StructuredFilterCleared,
    RenderedSearchQueryUpdated(String),
    RenderedSearchCleared,
    RenderedSearchPrevious,
    RenderedSearchNext,
    VisibleRowSelected {
        position: usize,
        request_scroll: bool,
    },
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

    pub(crate) fn apply_intent(&mut self, intent: DesktopIntent) {
        match intent {
            DesktopIntent::OpenFilesRequested => {
                self.state = DesktopAppState::Loading;
            }
            DesktopIntent::OpenFilesCancelled => {
                self.reset_idle();
            }
            DesktopIntent::LoadSucceeded(data) => {
                self.retained = Some(data);
                self.state = DesktopAppState::Loaded;
            }
            DesktopIntent::LoadFailed(message) => {
                self.retained = None;
                self.state = DesktopAppState::Error(message);
            }
            DesktopIntent::ResetRequested => {
                self.reset_idle();
            }
            DesktopIntent::StructuredFilterUpdated(filter) => {
                if let Some(data) = self.retained.as_mut() {
                    data.active_filter = filter;
                    data.rebuild_index();
                }
            }
            DesktopIntent::StructuredFilterCleared => {
                if let Some(data) = self.retained.as_mut() {
                    data.clear_filter();
                }
            }
            DesktopIntent::RenderedSearchQueryUpdated(query) => {
                if let Some(data) = self.retained.as_mut() {
                    data.set_rendered_search_query(query);
                }
            }
            DesktopIntent::RenderedSearchCleared => {
                if let Some(data) = self.retained.as_mut() {
                    data.set_rendered_search_query(String::new());
                }
            }
            DesktopIntent::RenderedSearchPrevious => {
                if let Some(data) = self.retained.as_mut() {
                    data.select_previous_rendered_match();
                }
            }
            DesktopIntent::RenderedSearchNext => {
                if let Some(data) = self.retained.as_mut() {
                    data.select_next_rendered_match();
                }
            }
            DesktopIntent::VisibleRowSelected {
                position,
                request_scroll,
            } => {
                if let Some(data) = self.retained.as_mut() {
                    data.select_visible_row(position, request_scroll);
                }
            }
        }
    }

    pub(crate) fn loaded_data(&self) -> Option<&RetainedDataSet> {
        self.retained.as_ref()
    }

    pub(crate) fn reset_idle(&mut self) {
        self.retained = None;
        self.state = DesktopAppState::Idle;
    }

    pub(crate) fn take_pending_scroll_to_selected(&mut self) -> bool {
        self.retained
            .as_mut()
            .map(|data| data.take_pending_scroll_to_selected())
            .unwrap_or(false)
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