use crate::error::{DataError, DataErrorKind};
use crate::marker::DataMarker;
use crate::request::DataLocale;
use crate::yoke::*;
use core::convert::TryFrom;
use core::marker::PhantomData;
use core::ops::Deref;
use alloc::rc::Rc as SelectedRc;
#[cfg(feature = "sync")]
use alloc::sync::Arc as SelectedRc;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataResponseMetadata {
    pub locale: Option<DataLocale>,
     buffer_format: Option<crate::buf::BufferFormat>,
}
pub struct DataPayload<M>
where
    M: DataMarker,
{
     yoke: Yoke<M::Yokeable, Option<Cart>>,
}
 struct Cart(SelectedRc<Box<[u8]>>);

impl<M> DataPayload<M>
where
    M: DataMarker,
{
    pub fn try_unwrap_owned(self) -> Result<M::Yokeable, DataError> {loop{}}
     fn try_map_project<M2, F, E>(self, f: F) -> Result<DataPayload<M2>, E>
    where
        M2: DataMarker,
        F: for<'a> FnOnce(
            <M::Yokeable as Yokeable<'a>>::Output,
            PhantomData<&'a ()>,
        ) -> Result<<M2::Yokeable as Yokeable<'a>>::Output, E>,
    {loop{}}
}
pub struct DataResponse<M>
where
    M: DataMarker,
{
    pub metadata: DataResponseMetadata,
    pub payload: Option<DataPayload<M>>,
}
