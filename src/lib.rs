enum Enum {
    A,
    B,
}
struct Error {
    foo: &'static (),
    bar: Enum,
}
enum Input {
    A,
    B,
    C(Error),
}
enum Res2 {
    Ok(Enum),
    Err(Error),
}
fn foo(other: Input) -> Res2 {
    match other {
        Input::A => Res2::Ok(Enum::A),
        Input::B => Res2::Ok(Enum::A),
        Input::C(foo) => Res2::Err(foo),
    }
}