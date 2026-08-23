#[derive(Debug)]
pub enum TiledError {
    NotImplemented,
    Xml(String),
    MissingField(&'static str),
}
