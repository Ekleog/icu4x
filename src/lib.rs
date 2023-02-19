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
    other.transpose()
}