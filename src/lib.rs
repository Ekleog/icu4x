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
    SomeOk,
    SomeErr(Error),
}
enum Opt2 {
    None,
    Some,
}
enum Res2 {
    Ok(Opt2),
    Err(Error),
}
fn foo(other: Opt1) -> Res2 {
    match other {
        Opt1::SomeOk => Res2::Ok(Opt2::Some),
        Opt1::SomeErr(foo) => Res2::Err(foo),
        Opt1::None => Res2::Ok(Opt2::None),
    }
}