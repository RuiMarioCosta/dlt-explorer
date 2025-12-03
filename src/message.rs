use iced::mouse::ScrollDelta;

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Expand,
    LoadFile,
    Loadfile(&'static str),
    Filter(String),
    Submitted,
    WindowResized(u32, u32),

    Scroll(ScrollDelta),
    ScrollBar(f32),
}
