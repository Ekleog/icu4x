enum Enum1 {
    A,
    B(&'static [()]),
    C(u8),
}
enum Enum2 {
    A,
    B,
}
struct Error {
    foo: Enum1,
    bar: Enum2,
}
fn foo(other: Option<Result<(), Error>>) -> Result<Option<()>, Error> {
    match other {
        Some(Ok(foo)) => Ok(Some(foo)),
        Some(Err(foo)) => Err(foo),
        None => Ok(None),
    }
}