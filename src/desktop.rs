use crate::dlt;
use crate::dlt::error::ParseError;
use anyhow::{Result, anyhow};
use eframe::egui;
use std::path::PathBuf;

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
}

#[derive(Debug)]
struct RetainedDataSet {
    paths: Vec<PathBuf>,
    version: u8,
    parse_errors: Vec<ParseError>,
    dlt: RetainedDlt,
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

    fn loaded_data(&self) -> Option<&RetainedDataSet> {
        self.retained.as_ref()
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
        Ok(RetainedDataSet {
            paths,
            version,
            parse_errors,
            dlt: RetainedDlt::V1(dlt),
        })
    } else if version == 2 {
        let (dlt, parse_errors) = dlt::v2::Dlt::open(paths.clone())?;
        Ok(RetainedDataSet {
            paths,
            version,
            parse_errors,
            dlt: RetainedDlt::V2(dlt),
        })
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

            match self.model.state() {
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

                    if let Some(data) = self.model.loaded_data() {
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
    use super::{DesktopAppState, DesktopModel, load_retained_dataset};
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
}
