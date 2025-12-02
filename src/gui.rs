use crate::cmd_line_parser;
use crate::dlt;
use crate::iconbar;
use crate::message;
use crate::toolbar;
use crate::viewer;

use std::path::PathBuf;

use cmd_line_parser::Cli;
use dlt::Dlt;

use iced::mouse::ScrollDelta;
use iced::widget::{Column, Container, button, column, container, row, text};
use iced::{Element, Length, Subscription, window};

use message::Message;
use viewer::table;

use iced::widget::MouseArea;

#[derive(Default)]
pub struct GUI<'a> {
    pub text: String,
    pub filter_buffer: String,
    pub file_content: String,
    pub width: u32,
    pub height: u32,
    pub dlts: Dlt<'a>,
    pub indexs: Vec<String>,

    pub location: f32,
    pub rows_per_view: usize,

    pub scroll: f32,
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

    pub fn visible_range(&self) -> std::ops::Range<usize> {
        let start = self.scroll as usize;

        start..start + self.rows_per_view
    }

    pub fn view(&self) -> Element<'_, Message> {
        let menubar = toolbar::get_toolbar();
        let iconbar = iconbar::get_iconbar(self);

        column![
            column![menubar, iconbar],
            row![
                button("100k").on_press(Message::Loadfile("/tests/data/testfile_100k_rows.dlt")),
                button("number_and_text").on_press(Message::Loadfile(
                    "/tests/data/testfile_number_and_text.dlt"
                )),
            ],
            table(self),
        ]
        .into()
    }

    pub fn update(&mut self, message: Message) {
        self.rows_per_view = 17;
        match message {
            Message::Loadfile(file) => {
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR").to_string() + file);
                println!("{:?}", path);
                self.dlts = Dlt::from_files(vec![path], None).unwrap();

                self.indexs = Vec::with_capacity(self.dlts.size());
                self.indexs = (0..self.dlts.size() as u32)
                    .map(|number| number.to_string())
                    .collect();

                // Scroll to the top
                self.scroll = 0.0;
            }
            Message::Scroll(delta) => {
                let dy = match delta {
                    // TODO: CHECK THIS. NOT SURE IF Y IS IN THE RIGHT PLACE
                    ScrollDelta::Lines { y, .. } => y,
                    ScrollDelta::Pixels { y, .. } => y,
                };

                // Scroll down
                if dy < 0.0 {
                    self.scroll += 1.0;
                }

                // Scroll up
                if dy > 0.0 {
                    self.scroll -= 1.0;
                }

                self.scroll = self.scroll.ceil();

                if self.dlts.size() >= self.rows_per_view {
                    self.scroll = self
                        .scroll
                        .clamp(0.0, (self.dlts.size() - self.rows_per_view) as f32);
                } else if self.dlts.size() > 0 {
                    self.scroll = self.scroll.clamp(0.0, self.rows_per_view as f32);
                }
            }
            Message::LoadFile => {
                // INFO: fn process_in_terminal
                let args = Cli {
                    paths: Some(vec![PathBuf::from(
                        env!("CARGO_MANIFEST_DIR").to_string()
                            // + "/tests/data/testfile_type_id_and_text.dlt",
                        + "/tests/data/testfile_100k_rows.dlt",
                        // + "/tests/data/testfile_number_and_text.dlt",
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
                self.filter_buffer = content;
            }
            Message::Submitted => {
                self.text = self.filter_buffer.clone();
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
