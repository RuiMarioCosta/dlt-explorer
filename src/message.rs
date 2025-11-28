use iced::mouse::ScrollDelta;
use iced::widget::scrollable;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Expand,
    LoadFile,
    Loadfile(&'static str),
    Filter(String),
    Submitted,
    WindowResized(u32, u32),
    Scrolled(scrollable::Viewport),

    ValueChanged(f32),
    Scroll(ScrollDelta),
}
