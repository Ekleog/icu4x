pub enum Enum {
    A,
    B,
}
pub struct Error {
    _foo: &'static (),
    _bar: Enum,
}
pub enum Input {
    A,
    B,
    C(Error),
}
pub enum Output {
    Ok(Enum),
    Err(Error),
}
pub fn foo(other: Input) -> Output {
    match other {
        Input::A => Output::Ok(Enum::A),
        Input::B => Output::Ok(Enum::A),
        Input::C(foo) => Output::Err(foo),
    }
}