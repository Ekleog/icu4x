fn try_from(other: Option<()>) -> Result<(), DataError> {
    other
        .map(|p| -> Result<(), DataError> { loop {} })
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
