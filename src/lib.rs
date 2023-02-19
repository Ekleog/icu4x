enum Enum {
    A,
    B,
}
struct Error {
    foo: &'static (),
    bar: Enum,
}
enum Opt1 {
    None,
    Some(Res1),
}
enum Res1 {
    Ok(()),
    Err(Error),
}
fn foo(other: Opt1) -> Result<Option<()>, Error> {
    match other {
        Opt1::Some(Res1::Ok(foo)) => Ok(Some(foo)),
        Opt1::Some(Res1::Err(foo)) => Err(foo),
        Opt1::None => Ok(None),
    }
}