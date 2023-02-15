#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum BufferFormat {
    Json,
    Bincode1,
    Postcard1,
}
