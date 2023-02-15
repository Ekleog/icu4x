extern crate alloc;
 mod any;
 mod buf;
 mod dynutil;
mod error;
mod key;
 mod marker;
mod request;
mod response;
 mod prelude {
    pub use crate::error::DataError;
    pub use crate::key::DataKey;
    pub use crate::marker::DataMarker;
    pub use crate::response::DataPayload;
    pub use crate::response::DataResponse;
    pub use crate::response::DataResponseMetadata;
    #[cfg(feature = "serde")]
     use ::AsDeserializingBufferProvider;
    pub use yoke;
}
pub use prelude::*;
