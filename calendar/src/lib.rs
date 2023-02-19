fn try_from(other: Option<()>) -> Result<(), Error> {
    let o: Option<Result<(), Error>> = match other {
        Some(_) => loop {},
        None => None,
    };
    o.transpose();
    loop {}
}
enum Enum1 {
    MissigDataKey,
    MismatchedType(&'static [()]),
    Io(u8),
}
enum Enum2 {
    A,
    B,
}
struct Error {
    foo: Enum1,
    bar: Enum2,
}
