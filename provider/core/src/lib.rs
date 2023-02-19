mod any {
    use crate::prelude::*;
    pub struct AnyPayload;
    struct AnyResponse {
        metadata: DataResponseMetadata,
        payload: Option<AnyPayload>,
    }
    impl TryFrom<DataResponse> for AnyResponse {
        type Error = DataError;
        fn try_from(other: DataResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                metadata: other.metadata,
                payload: other.payload.map(|p| p.try_unwrap_owned()).transpose()?,
            })
        }
    }
}
mod error {
    use crate::prelude::*;
     enum DataErrorKind {
        MissingDataKey,
        MissingLocale,
        NeedsLocale,
        ExtraneousLocale,
        FilteredResource,
        MismatchedType(&'static str),
        MissingPayload,
        InvalidState,
        Custom,
        Io(std::io::ErrorKind),
        MissingSourceData,
        UnavailableBufferFormat,
    }
    pub struct DataError {
        kind: DataErrorKind,
        key: Option<DataKey>,
    }
}
mod key {
    pub struct DataKey;
}
mod response {
    use crate::error::{DataError, };
    pub struct DataResponseMetadata;
    pub struct DataPayload;
    impl DataPayload {
        pub fn try_unwrap_owned(self) -> Result<crate::any::AnyPayload, DataError> {loop{}}
    }
    pub struct DataResponse {
        pub metadata: DataResponseMetadata,
        pub payload: Option<DataPayload>,
    }
}
mod prelude {
    pub use crate::error::DataError;
    pub use crate::key::DataKey;
    pub use crate::response::DataResponse;
    pub use crate::response::DataResponseMetadata;
    pub use yoke;
}
pub use prelude::*;
