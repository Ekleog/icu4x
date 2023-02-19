enum Enum1 {
    B(&'static ()),
}
enum Enum2 {
    A,
    B,
}
struct Error {
    foo: &'static (),
    bar: Enum2,
}
fn foo(other: Option<Result<(), Error>>) -> Result<Option<()>, Error> {
    match other {
        Some(Ok(foo)) => Ok(Some(foo)),
        Some(Err(foo)) => Err(foo),
        None => Ok(None),
    }
}