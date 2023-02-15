use crate::prelude::*;
use core::any::Any;
use yoke::trait_hack::YokeTraitHack;
use yoke::Yokeable;
use zerofrom::ZeroFrom;
#[cfg(not(feature = "sync"))]
use alloc::rc::Rc as SelectedRc;
#[cfg(feature = "sync")]
#[cfg(feature = "sync")]
 trait MaybeSendSync: Send + Sync {}
#[cfg(not(feature = "sync"))]
 trait MaybeSendSync {}
#[cfg(not(feature = "sync"))]
impl<T> MaybeSendSync for T {}
#[derive(Debug, Clone)]
enum AnyPayloadInner {
    StructRef(&'static dyn Any),
    #[cfg(not(feature = "sync"))]
    PayloadRc(SelectedRc<dyn Any>),
    #[cfg(feature = "sync")]
    PayloadRc,
}
#[derive(Debug, Clone, Yokeable)]
 struct AnyPayload {
    inner: AnyPayloadInner,
    type_name: &'static str,
}
#[allow(clippy::exhaustive_structs)]
 struct AnyMarker;
impl DataMarker for AnyMarker {
    type Yokeable = AnyPayload;
}
impl<M> crate::dynutil::UpcastDataPayload<M> for AnyMarker
where
    M: DataMarker + 'static,
    M::Yokeable: MaybeSendSync,
{
    #[inline]
    fn upcast(other: DataPayload<M>) -> DataPayload<AnyMarker> {loop{}}
}
impl AnyPayload {
     fn downcast<M>(self) -> Result<DataPayload<M>, DataError>
    where
        M: DataMarker + 'static,
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        M::Yokeable: MaybeSendSync,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
    {loop{}}
}
impl<M> DataPayload<M>
where
    M: DataMarker + 'static,
    M::Yokeable: MaybeSendSync,
{
     fn wrap_into_any_payload(self) -> AnyPayload {
        AnyPayload {
            inner: {loop{}},
            type_name: core::any::type_name::<M>(),
        }
    }
}
impl DataPayload<AnyMarker> {
     fn downcast<M>(self) -> Result<DataPayload<M>, DataError>
    where
        M: DataMarker + 'static,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        M::Yokeable: MaybeSendSync,
    {
        self.try_unwrap_owned()?.downcast()
    }
}
#[allow(clippy::exhaustive_structs)]
 struct AnyResponse {
     metadata: DataResponseMetadata,
     payload: Option<AnyPayload>,
}
impl TryFrom<DataResponse<AnyMarker>> for AnyResponse {
    type Error = DataError;
    #[inline]
    fn try_from(other: DataResponse<AnyMarker>) -> Result<Self, Self::Error> {
        Ok(Self {
            metadata: other.metadata,
            payload: other.payload.map(|p| p.try_unwrap_owned()).transpose()?,
        })
    }
}
impl AnyResponse {
     fn downcast<M>(self) -> Result<DataResponse<M>, DataError>
    where
        M: DataMarker + 'static,
        for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
        M::Yokeable: ZeroFrom<'static, M::Yokeable>,
        M::Yokeable: MaybeSendSync,
    {
        Ok(DataResponse {
            metadata: self.metadata,
            payload: self.payload.map(|p| p.downcast()).transpose()?,
        })
    }
}
