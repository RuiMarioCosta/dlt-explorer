use crate::dlt::Dlt;
use iced::widget::{button, column, container, row, scrollable, text, Row};
use iced::{Color, Element, Font, Length, Task};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct DltRow {
    index: usize,
    seconds: u32,
    microseconds: i32,
    ecu: String,
    apid: String,
    ctid: String,
    message_type: String,
    log_info: String,
    payload: String,
}

#[derive(Debug, Clone)]
pub(crate) enum FileResult {
    Loaded(Arc<Vec<DltRow>>),
    Cancelled,
    Error(String),
}

#[derive(Default)]
pub struct DltExplorer {
    rows: Vec<DltRow>,
    error: Option<String>,
    loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenFile,
    FileResult(FileResult),
}

const COL_INDEX: f32 = 60.0;
const COL_TIME: f32 = 160.0;
const COL_ECU: f32 = 80.0;
const COL_APID: f32 = 80.0;
const COL_CTID: f32 = 80.0;
const COL_TYPE: f32 = 100.0;
const COL_LOG: f32 = 100.0;

impl DltExplorer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => {
                self.loading = true;
                self.error = None;
                Task::perform(open_and_parse_file(), Message::FileResult)
            }
            Message::FileResult(result) => {
                self.loading = false;
                match result {
                    FileResult::Loaded(rows) => {
                        self.rows =
                            Arc::try_unwrap(rows).unwrap_or_else(|arc| (*arc).clone());
                        self.error = None;
                    }
                    FileResult::Cancelled => {}
                    FileResult::Error(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let open_button = if self.loading {
            button("Loading...")
        } else {
            button("Open DLT File").on_press(Message::OpenFile)
        };

        let status = if let Some(ref err) = self.error {
            text(format!("Error: {}", err))
        } else {
            text(format!("{} messages", self.rows.len()))
        };

        let toolbar = row![open_button, status].spacing(10).padding(10);

        let header = table_header();

        let content: Element<Message> = if self.rows.is_empty() {
            container(text(
                "No DLT file loaded. Click 'Open DLT File' to begin.",
            ))
            .padding(20)
            .width(Length::Fill)
            .into()
        } else {
            let rows: Vec<Element<Message>> =
                self.rows.iter().map(|r| table_row(r).into()).collect();
            scrollable(iced::widget::Column::with_children(rows))
                .height(Length::Fill)
                .into()
        };

        column![toolbar, header, content].into()
    }
}

async fn open_and_parse_file() -> FileResult {
    let Some(handle) = rfd::AsyncFileDialog::new()
        .add_filter("DLT files", &["dlt"])
        .pick_file()
        .await
    else {
        return FileResult::Cancelled;
    };

    let path = handle.path().to_path_buf();
    match Dlt::from_files(vec![path], None) {
        Ok(dlt) => FileResult::Loaded(Arc::new(dlt_to_rows(&dlt))),
        Err(e) => FileResult::Error(e.to_string()),
    }
}

fn dlt_to_rows(dlt: &Dlt) -> Vec<DltRow> {
    (0..dlt.size())
        .map(|i| DltRow {
            index: i,
            seconds: dlt.seconds()[i],
            microseconds: dlt.microseconds()[i],
            ecu: dlt.ecus()[i].clone(),
            apid: dlt.apids()[i].clone(),
            ctid: dlt.ctids()[i].clone(),
            message_type: dlt.message_types()[i].to_string(),
            log_info: dlt.log_infos()[i].to_string(),
            payload: dlt.payloads()[i].clone(),
        })
        .collect()
}

fn cell<'a>(content: String, width: f32) -> Element<'a, Message> {
    container(text(content).size(13))
        .width(width)
        .padding([2, 4])
        .into()
}

fn fill_cell<'a>(content: String) -> Element<'a, Message> {
    container(text(content).size(13))
        .width(Length::Fill)
        .padding([2, 4])
        .into()
}

fn header_cell<'a>(label: &str, width: f32) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(13)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            }),
    )
    .width(width)
    .padding([4, 4])
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.85, 0.85, 0.85))),
        ..container::Style::default()
    })
    .into()
}

fn header_fill_cell<'a>(label: &str) -> Element<'a, Message> {
    container(
        text(label.to_string())
            .size(13)
            .font(Font {
                weight: iced::font::Weight::Bold,
                ..Font::default()
            }),
    )
    .width(Length::Fill)
    .padding([4, 4])
    .style(|_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color::from_rgb(0.85, 0.85, 0.85))),
        ..container::Style::default()
    })
    .into()
}

fn table_header<'a>() -> Element<'a, Message> {
    Row::new()
        .push(header_cell("#", COL_INDEX))
        .push(header_cell("Time", COL_TIME))
        .push(header_cell("ECU", COL_ECU))
        .push(header_cell("App ID", COL_APID))
        .push(header_cell("Ctx ID", COL_CTID))
        .push(header_cell("Type", COL_TYPE))
        .push(header_cell("Log Level", COL_LOG))
        .push(header_fill_cell("Payload"))
        .into()
}

fn table_row<'a>(r: &DltRow) -> Element<'a, Message> {
    let bg_color = if r.index % 2 == 0 {
        Color::from_rgb(0.97, 0.97, 0.97)
    } else {
        Color::WHITE
    };

    let timestamp = format!("{}.{:06}", r.seconds, r.microseconds);

    container(
        Row::new()
            .push(cell(r.index.to_string(), COL_INDEX))
            .push(cell(timestamp, COL_TIME))
            .push(cell(r.ecu.clone(), COL_ECU))
            .push(cell(r.apid.clone(), COL_APID))
            .push(cell(r.ctid.clone(), COL_CTID))
            .push(cell(r.message_type.clone(), COL_TYPE))
            .push(cell(r.log_info.clone(), COL_LOG))
            .push(fill_cell(r.payload.clone())),
    )
    .style(move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(bg_color)),
        ..container::Style::default()
    })
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_explorer_has_no_rows() {
        let explorer = DltExplorer::default();
        assert!(explorer.rows.is_empty());
        assert!(explorer.error.is_none());
        assert!(!explorer.loading);
    }
}
