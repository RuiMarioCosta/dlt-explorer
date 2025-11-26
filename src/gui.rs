use crate::cmd_line_parser;
use crate::dlt;
use crate::iconbar;
use crate::message;
use crate::toolbar;
use crate::viewer;

use std::path::PathBuf;

use cmd_line_parser::Cli;
use dlt::Dlt;

use iced::widget::{Column, Container, Row, Space, container, text};
use iced::{Element, Length, Subscription, window};

use message::Message;
use viewer::table;

#[derive(Default)]
pub struct GUI<'a> {
    pub text: String,
    pub buffer: String,
    pub file_content: String,
    pub width: u32,
    pub height: u32,
    pub dlts: Dlt<'a>,
    pub indexs: Vec<String>,
}

impl<'a> GUI<'a> {
    // FIXME:Move this somewhere
    fn text_box(&self, content: &'a str) -> Element<'a, Message> {
        let mut width: f32 = self.width as f32;
        let mut height: f32 = self.height as f32;

        // width /= 2.0;
        // height -= 100.0;

        width -= 100.0;
        height /= 2.0;

        Container::new(text(content))
            .width(Length::Fixed(width))
            .height(Length::Fixed(height))
            .padding(50)
            .style(|theme| container::bordered_box(theme))
            .into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menubar = toolbar::get_toolbar();
        let iconbar = iconbar::get_iconbar(self);

        Column::new()
            .push(menubar)
            .push(iconbar)
            .push(text(format!(
                "Window Size: {} x {}",
                self.width, self.height
            )))
            .push(
                Row::new()
                    .spacing(50)
                    .push(Space::new(0.1, 0.0))
                    .push(Space::new(0.1, 0.0))
                    .push(table(self))
                    .push(Space::new(0.1, 0.0)),
            )
            .push(Space::new(0.0, 50.0))
            // TODO: DLT Search
            .push(self.text_box(&self.file_content))
            .into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::LoadFile => {
                // INFO: fn process_in_terminal
                let args = Cli {
                    paths: Some(vec![PathBuf::from(
                        env!("CARGO_MANIFEST_DIR").to_string()
                            // + "/tests/data/testfile_100k_rows.dlt",
                        + "/tests/data/testfile_number_and_text.dlt",
                    )]),
                    filter: None,
                    terminal: true,
                    sort: true,
                };

                let Some(mut paths) = args.paths else {
                    println!("ERROR IN PATH");
                    return;
                };

                if args.sort {
                    paths.sort();
                }

                self.dlts = Dlt::from_files(paths, args.filter).unwrap();
                // INFO: End of process_in_terminal

                self.indexs = Vec::with_capacity(self.dlts.size());
                self.indexs = (0..self.dlts.size() as u32)
                    .map(|number| number.to_string())
                    .collect();
            }
            Message::Filter(content) => {
                self.buffer = content;
            }
            Message::Submitted => {
                self.text = self.buffer.clone();
                println!("Final value = {}", self.text);
            }
            Message::WindowResized(width, height) => {
                self.width = width;
                self.height = height;
            }
            _ => {}
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        window::resize_events()
            .map(|(_, size)| Message::WindowResized(size.width as u32, size.height as u32))
    }
}
