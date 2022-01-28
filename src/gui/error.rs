pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Io,
    Fmt,
}
