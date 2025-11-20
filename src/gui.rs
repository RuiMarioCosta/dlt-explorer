use iced::alignment::Vertical;
use iced::border::Radius;
use iced::widget::{
    Column, Container, Image, Row, Text, Tooltip, button, column, container, image, row, text,
    text_input, tooltip,
};
use iced::{Alignment, Border, Color, Element, Length, Theme, theme};

use iced_aw::menu::Item;
use iced_aw::style::{Status, menu_bar::primary};
use iced_aw::widget::InnerBounds;
use iced_aw::{ContextMenu, Menu, MenuBar, menu, menu_bar, menu_items, quad};

use std::fmt::Display;

fn separator() -> quad::Quad {
    quad::Quad {
        quad_color: Color::from([0.5; 3]).into(),
        quad_border: Border {
            radius: Radius::new(4.0),
            ..Default::default()
        },
        inner_bounds: InnerBounds::Ratio(0.98, 0.2),
        height: Length::Fixed(5.0),
        ..Default::default()
    }
}

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

#[derive(Clone, Debug, Default)]
enum ToolBar {
    #[default]
    File,
    Search,
    Project,
    Config,
    Dlt,
    Filter,
    Plugin,
    View,
    Help,
}

#[derive(Debug, Clone)]
pub enum Message {
    Expand,
    Filter(String),
    Submitted,
}

#[derive(Default)]
pub struct DLT {
    last_message: Option<Message>,
    selected: ToolBar,
    text: String,
    buffer: String,
}

impl DLT {
    pub fn view(&self) -> Element<'_, Message> {
        let sub_menu = |items| Menu::new(items).width(180).offset(15.0).spacing(5.0);

        #[rustfmt::skip]
        let menubar = menu_bar!(
            (text(ToolBar::File.to_string()), sub_menu(menu_items!(
                // (tooltiper(button("New").on_press(Message::Expand).width(Length::Fill), "Hello World"))
                (text("New").align_y(Alignment::Start).width(Length::Fill))
                (text("Open").align_y(Alignment::Start).width(Length::Fill))
                (text("Save As").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Clear").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Recent files").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT Stream").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT Stream with Serial Header").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT from PCAP").align_y(Alignment::Start).width(Length::Fill))
                (text("Import IPC from PCAP").align_y(Alignment::Start).width(Length::Fill))
                (text("Append DLT file").align_y(Alignment::Start).width(Length::Fill))
                (text("Copy to clipboard").align_y(Alignment::Start).width(Length::Fill))
                (text("Export").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Settings").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Quit").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Search.to_string()), sub_menu(menu_items!(
                (text("Find...").align_y(Alignment::Start).width(Length::Fill))
                (text("Jump To...").align_y(Alignment::Start).width(Length::Fill))
                (text("History").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Project.to_string()), sub_menu(menu_items!(
                (text("New").align_y(Alignment::Start).width(Length::Fill))
                (text("Open").align_y(Alignment::Start).width(Length::Fill))
                (text("Save As").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Recent Projects").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Config.to_string()), sub_menu(menu_items!(
                (text("ECU Add").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Delete All Contexts").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("ECU Connect").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Diconnect").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Expand All ECUs").align_y(Alignment::Start).width(Length::Fill))
                (text("Collapse All ECUs").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Application Add").align_y(Alignment::Start).width(Length::Fill))
                (text("Application Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("Application Delete").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Context Add").align_y(Alignment::Start).width(Length::Fill))
                (text("Context Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("Context Delete").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Apply Configurations").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Dlt.to_string()), sub_menu(menu_items!(
                (text("Get Default Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Set Default Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Log Info").align_y(Alignment::Start).width(Length::Fill))
                (text("Set Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Set All Log Levels").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Store Config").align_y(Alignment::Start).width(Length::Fill))
                (text("Reset to Factory Default").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Send Injection").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Software Version").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Local Time").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Marker").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("ECU Edit All Log Levels").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Filter.to_string()), sub_menu(menu_items!(
                (text("Save Filter...").align_y(Alignment::Start).width(Length::Fill))
                (text("Load Filter...").align_y(Alignment::Start).width(Length::Fill))
                (text("Append Filter...").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Recent Filter").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Filter Add...").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Edit...").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Duplicate").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Clear All").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Reload Multifilter List").align_y(Alignment::Start).width(Length::Fill))
                (text("Refresh Multifilter Index").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Enable Filters").align_y(Alignment::Start).width(Length::Fill))
                (text("Sort By Time").align_y(Alignment::Start).width(Length::Fill))
                (text("Sort By Timestamp").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Plugin.to_string()), sub_menu(menu_items!(
                (text("Plugin Edit...").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Show").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Hide").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Disable").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Enable Plugins").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::View.to_string()), sub_menu(menu_items!(
                (text("Project").align_y(Alignment::Start).width(Length::Fill))
                (text("Search Results").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Help.to_string()), sub_menu(menu_items!(
                (text("Info...").align_y(Alignment::Start).width(Length::Fill))
                (text("Support...").align_y(Alignment::Start).width(Length::Fill))
                (separator())
                (text("Command Line Options...").align_y(Alignment::Start).width(Length::Fill))
            )))
        ).spacing(5)
        .draw_path(menu::DrawPath::Backdrop)
        .style(|theme:&iced::Theme, status: Status | menu::Style{
            // TODO: Check new color for hover here
            path_border: Border{
                radius: Radius::new(6.0),
                ..Default::default()
            },
            path: Color::from_rgb(
                theme.extended_palette().primary.weak.color.r * 1.2,
                theme.extended_palette().primary.weak.color.g * 1.2,
                theme.extended_palette().primary.weak.color.b * 1.2,
            ).into(),
            ..primary(theme, status)
        });

        let icon_bar = Row::new()
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
                text_input("", &self.buffer)
                    .on_input(Message::Filter)
                    .on_submit(Message::Submitted),
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
            ));

        Column::new().push(menubar).push(icon_bar).into()
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Filter(content) => {
                self.buffer = content;
            }
            Message::Submitted => {
                self.text = self.buffer.clone();
                println!("Final value = {}", self.text);
            }
            _ => {}
        }
    }
}

impl Display for ToolBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolBar::File => write!(f, "File"),
            ToolBar::Search => write!(f, "Search"),
            ToolBar::Project => write!(f, "Project"),
            ToolBar::Config => write!(f, "Config"),
            ToolBar::Dlt => write!(f, "Dlt"),
            ToolBar::Filter => write!(f, "Filter"),
            ToolBar::Plugin => write!(f, "Plugin"),
            ToolBar::View => write!(f, "View"),
            ToolBar::Help => write!(f, "Help"),
        }
    }
}
