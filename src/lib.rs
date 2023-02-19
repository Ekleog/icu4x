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
enum Output {
    Ok(Enum),
    Err(Error),
}
fn foo(other: Input) -> Output {
    match other {
        Input::A => Output::Ok(Enum::A),
        Input::B => Output::Ok(Enum::A),
        Input::C(foo) => Output::Err(foo),
    }
}