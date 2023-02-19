fn try_from(other: Option<()>) -> Result<(), DataError> {
    let o: Option<Result<(), DataError>> = match other {
        Some(o) => loop {},
        None => None,
    };
    o.transpose();
    loop {}
}
enum DataErrorKind {
    MissingDataKey,
    MismatchedType(&'static [()]),
    Io(u8),
}
enum Enum {
    A,
    B(()),
}
struct DataError {
    kind: DataErrorKind,
    key: Enum,
}
