enum Enum {
    A,
    B,
}
struct Error {
    foo: &'static (),
    bar: Enum,
}
enum Input {
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
fn foo(other: Input) -> Res2 {
    match other {
        Input::SomeOk => Res2::Ok(Opt2::Some),
        Input::SomeErr(foo) => Res2::Err(foo),
        Input::None => Res2::Ok(Opt2::None),
    }
}