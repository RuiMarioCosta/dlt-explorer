use iced::alignment::Vertical;
use iced::widget::{Column, Container, Row, button, column, row, text};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::menu::Item;
use iced_aw::{ContextMenu, Menu, MenuBar, menu, menu_bar, menu_items};

use std::fmt::Display;

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

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Expand,
}

#[derive(Default)]
pub struct Counter {
    last_message: Option<Message>,
    selected: ToolBar,
}

impl Counter {
    pub fn view(&self) -> Element<'_, Message> {
        let sub_menu = |items| Menu::new(items).width(180).offset(15.0).spacing(5.0);

        #[rustfmt::skip]
        let menubar = menu_bar!(
            (text(ToolBar::File.to_string()), sub_menu(menu_items!(
                (text("New").align_y(Alignment::Start).width(Length::Fill))
                (text("Open").align_y(Alignment::Start).width(Length::Fill))
                (text("Save As").align_y(Alignment::Start).width(Length::Fill))
                (text("Clear").align_y(Alignment::Start).width(Length::Fill))
                (text("Recent files").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT Stream").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT Stream with Serial Header").align_y(Alignment::Start).width(Length::Fill))
                (text("Import DLT from PCAP").align_y(Alignment::Start).width(Length::Fill))
                (text("Import IPC from PCAP").align_y(Alignment::Start).width(Length::Fill))
                (text("Append DLT file").align_y(Alignment::Start).width(Length::Fill))
                (text("Copy to clipboard").align_y(Alignment::Start).width(Length::Fill))
                (text("Export").align_y(Alignment::Start).width(Length::Fill))
                (text("Settings").align_y(Alignment::Start).width(Length::Fill))
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
                (text("Recent Projects").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Config.to_string()), sub_menu(menu_items!(
                (text("ECU Add").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Delete All Contexts").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Connect").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Diconnect").align_y(Alignment::Start).width(Length::Fill))
                (text("Expand All ECUs").align_y(Alignment::Start).width(Length::Fill))
                (text("Collapse All ECUs").align_y(Alignment::Start).width(Length::Fill))
                (text("Application Add").align_y(Alignment::Start).width(Length::Fill))
                (text("Application Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("Application Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("Context Add").align_y(Alignment::Start).width(Length::Fill))
                (text("Context Edit").align_y(Alignment::Start).width(Length::Fill))
                (text("Context Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("Apply Configurations").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Dlt.to_string()), sub_menu(menu_items!(
                (text("Get Default Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Set Default Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Log Info").align_y(Alignment::Start).width(Length::Fill))
                (text("Set Log Level").align_y(Alignment::Start).width(Length::Fill))
                (text("Set All Log Levels").align_y(Alignment::Start).width(Length::Fill))
                (text("Store Config").align_y(Alignment::Start).width(Length::Fill))
                (text("Reset to Factory Default").align_y(Alignment::Start).width(Length::Fill))
                (text("Send Injection").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Software Version").align_y(Alignment::Start).width(Length::Fill))
                (text("Get Local Time").align_y(Alignment::Start).width(Length::Fill))
                (text("Marker").align_y(Alignment::Start).width(Length::Fill))
                (text("ECU Edit All Log Levels").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Filter.to_string()), sub_menu(menu_items!(
                (text("Save Filter...").align_y(Alignment::Start).width(Length::Fill))
                (text("Load Filter...").align_y(Alignment::Start).width(Length::Fill))
                (text("Append Filter...").align_y(Alignment::Start).width(Length::Fill))
                (text("Recent Filter").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Add...").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Edit...").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Duplicate").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Delete").align_y(Alignment::Start).width(Length::Fill))
                (text("Filter Clear All").align_y(Alignment::Start).width(Length::Fill))
                (text("Reload Multifilter List").align_y(Alignment::Start).width(Length::Fill))
                (text("Refresh Multifilter Index").align_y(Alignment::Start).width(Length::Fill))
                (text("Enable Filters").align_y(Alignment::Start).width(Length::Fill))
                (text("Sort By Time").align_y(Alignment::Start).width(Length::Fill))
                (text("Sort By Timestamp").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Plugin.to_string()), sub_menu(menu_items!(
                (text("Plugin Edit...").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Show").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Hide").align_y(Alignment::Start).width(Length::Fill))
                (text("Plugin Disable").align_y(Alignment::Start).width(Length::Fill))
                (text("Enable Plugins").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::View.to_string()), sub_menu(menu_items!(
                (text("Project").align_y(Alignment::Start).width(Length::Fill))
                (text("Search Results").align_y(Alignment::Start).width(Length::Fill))
            )))
            (text(ToolBar::Help.to_string()), sub_menu(menu_items!(
                (text("Info...").align_y(Alignment::Start).width(Length::Fill))
                (text("Support...").align_y(Alignment::Start).width(Length::Fill))
                (text("Command Line Options...").align_y(Alignment::Start).width(Length::Fill))
            )))
        ).spacing(5);

        menubar.into()
    }

    pub fn update(&mut self, message: Message) {
        self.last_message = Some(message);
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
