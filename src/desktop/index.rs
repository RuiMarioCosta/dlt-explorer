use crate::desktop::retained::{LogTableRow, RetainedDlt, StructuredFilter};
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub(crate) struct QueryPipeline<'a> {
    structured_filter: &'a StructuredFilter,
    rendered_search_query: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexLayer {
    pub(crate) visible_indices: Vec<usize>,
    rendered_search_match_positions: Vec<usize>,
}

impl<'a> QueryPipeline<'a> {
    pub(crate) fn new(
        structured_filter: &'a StructuredFilter,
        rendered_search_query: &'a str,
    ) -> Self {
        Self {
            structured_filter,
            rendered_search_query,
        }
    }

    pub(crate) fn build(self, dlt: &RetainedDlt) -> IndexLayer {
        let structured_filtered_indices = self.apply_structured_filter(dlt);
        let visible_indices = self.apply_rendered_text_search(dlt, structured_filtered_indices);
        let rendered_search_match_positions = if self.rendered_search_query.is_empty() {
            Vec::new()
        } else {
            (0..visible_indices.len()).collect()
        };

        IndexLayer {
            visible_indices,
            rendered_search_match_positions,
        }
    }

    fn apply_structured_filter(self, dlt: &RetainedDlt) -> Vec<usize> {
        (0..dlt.len())
            .filter(|&index| {
                let ecu_matches = contains_ignore_case(
                    dlt.ecu(index),
                    self.structured_filter.ecu_contains.as_str(),
                );
                let apid_matches = contains_ignore_case(
                    dlt.apid(index),
                    self.structured_filter.apid_contains.as_str(),
                );
                let ctid_matches = contains_ignore_case(
                    dlt.ctid(index),
                    self.structured_filter.ctid_contains.as_str(),
                );
                let kind = super::retained::format_message_type(
                    dlt.message_type(index),
                    dlt.message_type_info(index),
                );
                let kind_matches = contains_ignore_case(
                    kind.as_str(),
                    self.structured_filter.kind_contains.as_str(),
                );

                ecu_matches && apid_matches && ctid_matches && kind_matches
            })
            .collect()
    }

    fn apply_rendered_text_search(
        self,
        dlt: &RetainedDlt,
        structured_filtered_indices: Vec<usize>,
    ) -> Vec<usize> {
        if self.rendered_search_query.is_empty() {
            return structured_filtered_indices;
        }

        structured_filtered_indices
            .into_iter()
            .filter(|&index| {
                let rendered = dlt.rendered_row_text(index);
                contains_ignore_case(rendered.as_str(), self.rendered_search_query)
            })
            .collect()
    }
}

impl IndexLayer {
    pub(crate) fn empty() -> Self {
        Self {
            visible_indices: Vec::new(),
            rendered_search_match_positions: Vec::new(),
        }
    }

    pub(crate) fn from_filter_and_search(
        dlt: &RetainedDlt,
        filter: &StructuredFilter,
        rendered_search_query: &str,
    ) -> Self {
        QueryPipeline::new(filter, rendered_search_query).build(dlt)
    }

    pub(crate) fn visible_count(&self) -> usize {
        self.visible_indices.len()
    }

    pub(crate) fn visible_rows(
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

    pub(crate) fn visible_index_at(&self, position: usize) -> Option<usize> {
        self.visible_indices.get(position).copied()
    }

    pub(crate) fn position_for_index(&self, index: usize) -> Option<usize> {
        self.visible_indices.iter().position(|&value| value == index)
    }

    pub(crate) fn rendered_search_match_count(&self) -> usize {
        self.rendered_search_match_positions.len()
    }

    pub(crate) fn rendered_search_match_position(&self, match_index: usize) -> Option<usize> {
        self.rendered_search_match_positions.get(match_index).copied()
    }

    pub(crate) fn rendered_search_match_ordinal(
        &self,
        active_match_position: usize,
    ) -> Option<usize> {
        self.rendered_search_match_positions
            .iter()
            .position(|&pos| pos == active_match_position)
            .map(|idx| idx + 1)
    }

    pub(crate) fn is_rendered_search_match_position(&self, position: usize) -> bool {
        self.rendered_search_match_positions.contains(&position)
    }
}

fn contains_ignore_case(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    value.to_ascii_lowercase().contains(&query.to_ascii_lowercase())
}