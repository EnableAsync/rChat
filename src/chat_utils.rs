use serde_cbor::Error;
use std::io;

pub fn to_io_error(err: Error) -> io::Error {
    use std::error::Error;
    use serde_cbor::error::Category;

    let kind = match err.classify() {
        Category::Syntax => io::ErrorKind::InvalidInput,
        Category::Data => io::ErrorKind::InvalidData,
        Category::Eof => io::ErrorKind::UnexpectedEof,
        Category::Io => err
            .source()
            .and_then(|e| e.downcast_ref::<std::io::Error>())
            .map(|e| e.kind())
            .unwrap_or(io::ErrorKind::Other),
    };
    io::Error::new(kind, err)
}