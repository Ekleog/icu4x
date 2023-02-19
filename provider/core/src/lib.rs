pub struct AnyPayload;
struct AnyResponse;
fn try_from(other: Option<()>) -> Result<AnyResponse, DataError> {
    other
        .map(|p| -> Result<AnyPayload, DataError> { loop {} })
        .transpose();
    loop {}
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
