use crate::GUI;
use crate::message;

use iced::widget::{Container, Scrollable, column, row, text};
use iced::{Element, Length};

use message::Message;

enum Viewer {
    Index,
    Time,
    Timestamp,
    Ecuid,
    Apid,
    Ctid,
    Type,
    Payload,
}

fn cell<'a>(content: &'a str, size: Length) -> Container<'a, Message> {
    Container::new(text(content)).padding(5).width(size)
}

pub fn table<'a>(entity: &'a GUI) -> Element<'a, Message> {
    let mut indexs = column!(cell(Viewer::Index.as_str(), Length::Fixed(150.0)));
    let times = column!(cell(Viewer::Time.as_str(), Length::Fixed(150.0)));
    let timestamps = column!(cell(Viewer::Timestamp.as_str(), Length::Fixed(150.0)));
    let ecus = column!(cell(Viewer::Ecuid.as_str(), Length::Fixed(150.0)));
    let mut apids = column!(cell(Viewer::Apid.as_str(), Length::Fixed(150.0)));
    let mut ctids = column!(cell(Viewer::Ctid.as_str(), Length::Fixed(150.0)));
    let types = column!(cell(Viewer::Type.as_str(), Length::Fixed(150.0)));
    let mut payloads = column!(cell(Viewer::Payload.as_str(), Length::Fill));

    for i in 0..entity.dlts.size() {
        indexs = indexs.push(cell(&entity.indexs[i], Length::Fixed(150.0)));
        apids = apids.push(cell(&entity.dlts.apids()[i], Length::Fixed(150.0)));
        // ctids = ctids.push(cell(&entity.dlts.ctids()[i], Length::Fixed(150.0)));
        // payloads = payloads.push(cell(&entity.dlts.payloads()[i], Length::Fill));
    }

    let items = row!(
        indexs, times, timestamps, ecus, apids, ctids, types, payloads
    );

    let mut height: f32 = entity.height as f32;
    // height /= 2.0;

    Scrollable::new(items)
        .height(Length::Fixed(height))
        .width(Length::Fill)
        .on_scroll(Message::Scrolled)
        .into()
}

impl Viewer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Viewer::Index => "Index",
            Viewer::Time => "Time",
            Viewer::Timestamp => "Timestamp",
            Viewer::Ecuid => "Ecuid",
            Viewer::Apid => "Apid",
            Viewer::Ctid => "Ctid",
            Viewer::Type => "Type",
            Viewer::Payload => "Payload",
        }
    }
}
