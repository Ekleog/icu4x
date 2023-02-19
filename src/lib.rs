enum Enum {
    A,
    B,
}
struct Error {
    foo: &'static (),
    bar: Enum,
}
fn foo(other: Option<Result<(), Error>>) -> Result<Option<()>, Error> {
    match other {
        Some(Ok(foo)) => Ok(Some(foo)),
        Some(Err(foo)) => Err(foo),
        None => Ok(None),
    }
}