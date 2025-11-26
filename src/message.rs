#[derive(Debug, Clone)]
pub enum Message {
    None,
    Expand,
    LoadFile,
    Filter(String),
    Submitted,
    WindowResized(u32, u32),
}
