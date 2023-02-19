
pub struct AnyPayload;
struct AnyResponse {
    payload: Option<AnyPayload>,
}
impl AnyResponse {
    fn try_from(other: Option<()>) -> Result<Self, DataError> {
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
pub use yoke;
