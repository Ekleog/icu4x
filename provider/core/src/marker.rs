use ::yoke::Yokeable;
pub trait DataMarker {
    type Yokeable: for<'a> Yokeable<'a>;
}
