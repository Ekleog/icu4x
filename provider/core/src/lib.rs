
pub struct AnyPayload;
struct AnyResponse {
    payload: Option<AnyPayload>,
}
impl AnyResponse {
    fn try_from(other: Option<DataPayload>) -> Result<Self, DataError> {
        other.map(|p| -> Result<AnyPayload, DataError> { loop {} }).transpose();
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
pub struct DataResponse {
    pub payload: Option<DataPayload>,
}
pub use yoke;
