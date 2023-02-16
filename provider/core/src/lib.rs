mod any {
    use crate::prelude::*;
    enum AnyPayloadInner {
        StructRef,
        PayloadRc,
    }
    pub struct AnyPayload;
    struct AnyMarker;
    struct AnyResponse {
        metadata: DataResponseMetadata,
        payload: Option<AnyPayload>,
    }
    impl TryFrom<DataResponse<AnyMarker>> for AnyResponse {
        type Error = DataError;
        fn try_from(other: DataResponse<AnyMarker>) -> Result<Self, Self::Error> {
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
        str_context: Option<&'static str>,
    }
}
mod key {
    pub struct DataKey;
}
mod response {
    use crate::error::{DataError, };
    use core::marker::PhantomData;
    pub struct DataResponseMetadata;
    pub struct DataPayload<M> {
        _foo: PhantomData<M>,
    }
    impl<M> DataPayload<M> {
        pub fn try_unwrap_owned(self) -> Result<crate::any::AnyPayload, DataError> {loop{}}
    }
    pub struct DataResponse<M> {
        pub metadata: DataResponseMetadata,
        pub payload: Option<DataPayload<M>>,
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
