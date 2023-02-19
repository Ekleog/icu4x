
pub struct AnyPayload;
struct AnyResponse {
    payload: Option<AnyPayload>,
}
impl AnyResponse {
    fn try_from(other: DataResponse) -> Result<Self, DataError> {
        other.payload.map(|p| -> Result<AnyPayload, DataError> { loop {} }).transpose();
        loop {}
    }
}
enum DataErrorKind {
    MissingDataKey,
    MismatchedType(&'static [()]),
    Io(u8),
}
pub struct DataError {
    kind: DataErrorKind,
    key: Option<()>,
}
pub struct DataResponseMetadata;
pub struct DataPayload;
impl DataPayload {
    pub fn try_unwrap_owned(self) -> Result<AnyPayload, DataError> {
        loop {}
    }
}
pub struct DataResponse {
    pub payload: Option<DataPayload>,
}
pub use yoke;
