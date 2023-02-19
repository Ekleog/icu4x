
pub struct AnyPayload;
struct AnyResponse;
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
pub use yoke;
