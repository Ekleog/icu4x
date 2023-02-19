fn try_from(other: Option<()>) -> Result<(), Error> {
    let o: Option<Result<(), Error>> = match other {
        Some(_) => loop {},
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
    B,
}
struct Error {
    foo: DataErrorKind,
    bar: Enum,
}
