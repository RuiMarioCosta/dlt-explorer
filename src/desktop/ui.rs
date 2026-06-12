use crate::desktop::application::{DesktopAppState, DesktopModel};
use crate::desktop::retained::{RetainedDataSet, load_retained_dataset};
use anyhow::{Result, anyhow};
use eframe::egui;

const TABLE_COL_TIMESTAMP: f32 = 140.0;
const TABLE_COL_ECU: f32 = 70.0;
const TABLE_COL_APID: f32 = 70.0;
const TABLE_COL_CTID: f32 = 70.0;
const TABLE_COL_TYPE: f32 = 140.0;
const TABLE_ROW_HEIGHT: f32 = 20.0;

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
        let query = data.rendered_search_query_mut().clone();
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