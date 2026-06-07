use anyhow::{Result, anyhow};
use eframe::egui;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DesktopAppState {
    Idle,
    Loading,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DesktopModel {
    state: DesktopAppState,
}

impl Default for DesktopModel {
    fn default() -> Self {
        Self {
            state: DesktopAppState::Idle,
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

    pub(crate) fn loading_failed(&mut self, message: impl Into<String>) {
        self.state = DesktopAppState::Error(message.into());
    }

    pub(crate) fn reset_idle(&mut self) {
        self.state = DesktopAppState::Idle;
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
                if ui.button("Start loading").clicked() {
                    self.model.begin_loading();
                }

                if ui.button("Set error").clicked() {
                    self.model.loading_failed("Failed to read DLT input");
                }

                if ui.button("Reset").clicked() {
                    self.model.reset_idle();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DLT Explorer");
            ui.separator();

            match self.model.state() {
                DesktopAppState::Idle => {
                    ui.label("State: idle");
                    ui.label("No file is being loaded.");
                }
                DesktopAppState::Loading => {
                    ui.label("State: loading");
                    ui.spinner();
                    ui.label("Loading DLT data...");
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
    use super::{DesktopAppState, DesktopModel};

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
}
