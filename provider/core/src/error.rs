use crate::buf::BufferFormat;
use crate::prelude::*;
use displaydoc::Display;
#[derive(Clone, Copy, Eq, PartialEq, Display, )]
pub enum DataErrorKind {
    #[displaydoc("Missing data for key")]
    MissingDataKey,
    #[displaydoc("Missing data for locale")]
    MissingLocale,
    #[displaydoc("Request needs a locale")]
    NeedsLocale,
    #[displaydoc("Request has an extraneous locale")]
    ExtraneousLocale,
    #[displaydoc("Resource blocked by filter")]
    FilteredResource,
    #[displaydoc("Mismatched types: tried to downcast with {0}, but actual type is different")]
    MismatchedType(&'static str),
    #[displaydoc("Missing payload")]
    MissingPayload,
    #[displaydoc("Invalid state")]
    InvalidState,
    #[displaydoc("Custom")]
    Custom,
    #[displaydoc("I/O error: {0:?}")]
    Io(std::io::ErrorKind),
    #[displaydoc("Missing source data")]
    MissingSourceData,
    #[displaydoc("Unavailable buffer format: {0:?} (does icu_provider need to be compiled with an additional Cargo feature?)")]
    UnavailableBufferFormat(BufferFormat),
}
pub struct DataError {
     kind: DataErrorKind,
     key: Option<DataKey>,
     str_context: Option<&'static str>,
}
