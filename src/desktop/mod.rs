mod application;
mod index;
mod retained;
mod ui;

pub use application::DesktopBenchmarkHarness;
pub(crate) use ui::run_desktop_shell;

#[cfg(test)]
mod tests {
    use super::application::{DesktopAppState, DesktopIntent, DesktopModel};
    use super::retained::{StructuredFilter, format_message_type, load_retained_dataset};
    use std::path::PathBuf;

    #[test]
    fn desktop_model_starts_idle() {
        let model = DesktopModel::default();
        assert_eq!(model.state(), &DesktopAppState::Idle);
    }

    #[test]
    fn desktop_model_transitions_loading_and_error() {
        let mut model = DesktopModel::default();

        model.apply_intent(DesktopIntent::OpenFilesRequested);
        let generation = model
            .active_load_generation()
            .expect("load generation should exist");
        assert_eq!(model.state(), &DesktopAppState::Loading);

        model.apply_intent(DesktopIntent::LoadFailed {
            generation,
            message: "boom".to_string(),
        });
        assert_eq!(model.state(), &DesktopAppState::Error("boom".to_string()));

        model.reset_idle();
        assert_eq!(model.state(), &DesktopAppState::Idle);
    }

    #[test]
    fn desktop_model_applies_core_state_intents_deterministically() {
        let mut model = DesktopModel::default();

        model.apply_intent(DesktopIntent::OpenFilesRequested);
        assert_eq!(model.state(), &DesktopAppState::Loading);

        model.apply_intent(DesktopIntent::OpenFilesCancelled);
        assert_eq!(model.state(), &DesktopAppState::Idle);

        model.apply_intent(DesktopIntent::OpenFilesRequested);
        let generation = model
            .active_load_generation()
            .expect("load generation should exist");
        model.apply_intent(DesktopIntent::LoadFailed {
            generation,
            message: "boom".to_string(),
        });
        assert_eq!(model.state(), &DesktopAppState::Error("boom".to_string()));

        model.apply_intent(DesktopIntent::ResetRequested);
        assert_eq!(model.state(), &DesktopAppState::Idle);
    }

    #[test]
    fn desktop_model_routes_loaded_mutations_through_intents() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut model = DesktopModel::default();
        let data = load_retained_dataset(vec![path]).expect("fixture should load");
        let total = data.message_count();

        model.apply_intent(DesktopIntent::OpenFilesRequested);
        let generation = model
            .active_load_generation()
            .expect("load generation should exist");
        model.apply_intent(DesktopIntent::LoadSucceeded { generation, data });
        assert_eq!(model.state(), &DesktopAppState::Loaded);

        model.apply_intent(DesktopIntent::StructuredFilterUpdated(StructuredFilter {
            kind_contains: "control".to_string(),
            ..StructuredFilter::default()
        }));

        let filtered_count = model
            .loaded_data()
            .map(|loaded| loaded.visible_message_count())
            .expect("data should stay loaded");
        assert!(filtered_count > 0);
        assert!(filtered_count <= total);

        model.apply_intent(DesktopIntent::RenderedSearchQueryUpdated(
            "no-such-rendered-text-token".to_string(),
        ));

        let post_query_count = model
            .loaded_data()
            .map(|loaded| loaded.visible_message_count())
            .expect("data should stay loaded");
        assert_eq!(post_query_count, 0);
    }

    #[test]
    fn desktop_model_ignores_stale_load_completion_events() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "tests/data/testfile_control_messages.dlt",
        );

        let mut model = DesktopModel::default();
        model.apply_intent(DesktopIntent::OpenFilesRequested);
        let stale_generation = model
            .active_load_generation()
            .expect("first load generation should exist");

        model.apply_intent(DesktopIntent::OpenFilesRequested);
        let active_generation = model
            .active_load_generation()
            .expect("second load generation should exist");

        let stale_data = load_retained_dataset(vec![path.clone()]).expect("fixture should load");
        model.apply_intent(DesktopIntent::LoadSucceeded {
            generation: stale_generation,
            data: stale_data,
        });

        assert_eq!(model.state(), &DesktopAppState::Loading);
        assert!(model.loaded_data().is_none());

        let active_data = load_retained_dataset(vec![path]).expect("fixture should load");
        model.apply_intent(DesktopIntent::LoadSucceeded {
            generation: active_generation,
            data: active_data,
        });

        assert_eq!(model.state(), &DesktopAppState::Loaded);
        assert!(model.loaded_data().is_some());
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
            let rendered = data.rendered_row_text_for_index(row.index);
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
        let selected_timestamp = data.visible_rows(selected_position..(selected_position + 1))[0]
            .timestamp
            .clone();

        data.set_rendered_search_query(selected_timestamp);

        assert_eq!(data.selected_row_index(), Some(selected_index));
        assert!(data.visible_message_count() > 0);
    }
}
