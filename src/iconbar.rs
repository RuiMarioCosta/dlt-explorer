use crate::GUI;
use crate::message;

use iced::widget::{Row, button, container, image, text, text_input, tooltip};
use iced::{Border, Color, Element, Length};

use message::Message;

fn tooltiper<'a>(
    content: impl Into<Element<'a, Message, iced::Theme, iced::Renderer>>,
    tooltip_text: &'a str,
) -> Element<'a, Message, iced::Theme, iced::Renderer> {
    tooltip(
        content,
        container(text(tooltip_text).color(Color::WHITE))
            .padding(10)
            .style(|theme| {
                container::rounded_box(theme)
                    .border(Border::default().rounded(8.0))
                    .background(Color::from_rgb(0.2, 0.2, 0.2))
            }),
        tooltip::Position::Bottom,
    )
    .into()
}

pub fn get_iconbar(entity: &GUI) -> iced::widget::Row<'static, message::Message> {
    let mut width: f32 = entity.width as f32;
    width /= 3.0;

    Row::new()
        .spacing(10)
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/document-new.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Create a new DLT file",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/document-open.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Open existing DLT file",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/edit-clear.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Create a new temporary DLT file and clear the screen",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/document-save-as.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Save current log as DLT file",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/document-save-as2.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Save current project as DLP file",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path(
                    "src/png/network-transmit-receive.png",
                ))
                .width(32)
                .height(32),
            )
            .on_press(Message::Expand),
            "Connect all ECU's or create a new one",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/network-offline.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Disconnect all connected ECU's",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/preferences-desktop.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Settings",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/go-bottom.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Scroll automatically to the end of the log, when receiving data",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/bookmark-new.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Write Marker into DLT file",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/weather-storm.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Apply Configurations",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/view-filter_32_on.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Toggle Filters Enabled/Disabled",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path(
                    "src/png/x-kde-nsplugin-generated.png",
                ))
                .width(32)
                .height(32),
            )
            .on_press(Message::Expand),
            "Toggle Plugins Enabled/Disabled",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/system-search.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Search for DLT message",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/action-regexp.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Use Regular Expressions when searching",
        ))
        .push(
            text_input("", &entity.filter_buffer)
                .on_input(Message::Filter)
                .on_submit(Message::Submitted)
                .width(Length::Fixed(width)),
        )
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/go-previous.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Search for the previous occurance",
        ))
        .push(tooltiper(
            button(
                image(image::Handle::from_path("src/png/go-next.png"))
                    .width(32)
                    .height(32),
            )
            .on_press(Message::Expand),
            "Search for the next occurance",
        ))
}
