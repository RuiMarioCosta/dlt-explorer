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
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Search.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Project.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Config.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Dlt.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Filter.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Plugin.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::View.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
            )))
            (text(ToolBar::Help.to_string()), sub_menu(menu_items!(
                (text("Open").align_y(Alignment::Start))
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
