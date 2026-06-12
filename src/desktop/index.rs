use crate::desktop::retained::{LogTableRow, RetainedDlt, StructuredFilter};
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IndexLayer {
    pub(crate) visible_indices: Vec<usize>,
}

impl IndexLayer {
    pub(crate) fn from_filter_and_search(
        dlt: &RetainedDlt,
        filter: &StructuredFilter,
        rendered_search_query: &str,
    ) -> Self {
        let structured_filtered_indices: Vec<usize> = (0..dlt.len())
            .filter(|&index| {
                let ecu_matches =
                    contains_ignore_case(dlt.ecu(index), filter.ecu_contains.as_str());
                let apid_matches = contains_ignore_case(
                    dlt.apid(index),
                    filter.apid_contains.as_str(),
                );
                let ctid_matches = contains_ignore_case(
                    dlt.ctid(index),
                    filter.ctid_contains.as_str(),
                );
                let kind =
                    super::retained::format_message_type(dlt.message_type(index), dlt.message_type_info(index));
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
}

fn contains_ignore_case(value: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    value.to_ascii_lowercase().contains(&query.to_ascii_lowercase())
}