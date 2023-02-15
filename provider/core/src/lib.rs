extern crate alloc;
mod any {
    use crate::prelude::*;
    use alloc::rc::Rc as SelectedRc;
    use core::any::Any;
    use yoke::trait_hack::YokeTraitHack;
    use yoke::Yokeable;
    use zerofrom::ZeroFrom;
    trait MaybeSendSync {}
    impl<T> MaybeSendSync for T {}
    #[derive(Debug, Clone)]
    enum AnyPayloadInner {
        StructRef(&'static dyn Any),
        PayloadRc(SelectedRc<dyn Any>),
    }
    #[derive(Debug, Clone, Yokeable)]
    struct AnyPayload {
        inner: AnyPayloadInner,
        type_name: &'static str,
    }
    struct AnyMarker;
    impl DataMarker for AnyMarker {
        type Yokeable = AnyPayload;
    }
    impl<M> crate::dynutil::UpcastDataPayload<M> for AnyMarker
    where
        M: DataMarker + 'static,
        M::Yokeable: MaybeSendSync,
    {
        fn upcast(other: DataPayload<M>) -> DataPayload<AnyMarker> {
            loop {}
        }
    }
    impl AnyPayload {
        fn downcast<M>(self) -> Result<DataPayload<M>, DataError>
        where
            M: DataMarker + 'static,
            M::Yokeable: ZeroFrom<'static, M::Yokeable>,
            M::Yokeable: MaybeSendSync,
            for<'a> YokeTraitHack<<M::Yokeable as Yokeable<'a>>::Output>: Clone,
        {
            loop {}
        }
    }
    impl<M> DataPayload<M>
    where
        M: DataMarker + 'static,
        M::Yokeable: MaybeSendSync,
    {
        fn wrap_into_any_payload(self) -> AnyPayload {
            AnyPayload {
                inner: { loop {} },
                type_name: core::any::type_name::<M>(),
            }
        }
    }
    struct AnyResponse {
        metadata: DataResponseMetadata,
        payload: Option<AnyPayload>,
    }
    impl TryFrom<DataResponse<AnyMarker>> for AnyResponse {
        type Error = DataError;
        fn try_from(other: DataResponse<AnyMarker>) -> Result<Self, Self::Error> {
            Ok(Self {
                metadata: other.metadata,
                payload: other.payload.map(|p| p.try_unwrap_owned()).transpose()?,
            })
        }
    }
}
mod buf {
    #[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
    pub enum BufferFormat {
        Json,
        Bincode1,
        Postcard1,
    }
}
mod dynutil {
    pub trait UpcastDataPayload<M>
    where
        M: crate::DataMarker,
        Self: Sized + crate::DataMarker,
    {
        fn upcast(other: crate::DataPayload<M>) -> crate::DataPayload<Self> {
            loop {}
        }
    }
}
mod error {
    use crate::buf::BufferFormat;
    use crate::prelude::*;
    use displaydoc::Display;
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub enum DataErrorKind {
        MissingDataKey,
        MissingLocale,
        NeedsLocale,
        ExtraneousLocale,
        FilteredResource,
        MismatchedType(&'static str),
        MissingPayload,
        InvalidState,
        Custom,
        Io(std::io::ErrorKind),
        MissingSourceData,
        UnavailableBufferFormat(BufferFormat),
    }
    pub struct DataError {
        kind: DataErrorKind,
        key: Option<DataKey>,
        str_context: Option<&'static str>,
    }
}
mod key {
    use core::ops::Deref;
    use writeable::{LengthHint, };
    use zerovec::ule::*;
    struct DataKeyHash([u8; 4]);
    enum FallbackPriority {
        Language,
        Region,
        Collation,
    }
    #[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd)]
    enum FallbackSupplement {
        Collation,
    }
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd)]
    struct DataKeyPath {
        tagged: &'static str,
    }
    impl DataKeyPath {
        const fn get(self) -> &'static str {
            loop {}
        }
    }
    struct DataKeyMetadata {
        fallback_priority: FallbackPriority,
        extension_key: Option<icu_locid::extensions::unicode::Key>,
        fallback_supplement: Option<FallbackSupplement>,
    }
    impl Default for DataKeyMetadata {
        fn default() -> Self {
            loop {}
        }
    }
    pub struct DataKey {
        path: DataKeyPath,
        hash: DataKeyHash,
        metadata: DataKeyMetadata,
    }
}
mod marker {
    use yoke::Yokeable;
    pub trait DataMarker {
        type Yokeable: for<'a> Yokeable<'a>;
    }
}
mod request {
    use core::fmt;
    use icu_locid::extensions::unicode as unicode_ext;
    use icu_locid::{LanguageIdentifier, Locale, SubtagOrderingResult};
    #[derive(PartialEq, Clone, Default, Eq, Hash)]
    pub struct DataLocale {
        langid: LanguageIdentifier,
        keywords: unicode_ext::Keywords,
    }
    impl fmt::Debug for DataLocale {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            loop {}
        }
    }
}
mod response {
    use crate::error::{DataError, DataErrorKind};
    use crate::marker::DataMarker;
    use crate::request::DataLocale;
    use crate::yoke::*;
    use alloc::rc::Rc as SelectedRc;
    use core::convert::TryFrom;
    use core::marker::PhantomData;
    use core::ops::Deref;
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DataResponseMetadata {
        pub locale: Option<DataLocale>,
        buffer_format: Option<crate::buf::BufferFormat>,
    }
    pub struct DataPayload<M>
    where
        M: DataMarker,
    {
        _foo: PhantomData<M>,
    }
    struct Cart(SelectedRc<Box<[u8]>>);
    impl<M> DataPayload<M>
    where
        M: DataMarker,
    {
        pub fn try_unwrap_owned(self) -> Result<M::Yokeable, DataError> {
            loop {}
        }
        fn try_map_project<M2, F, E>(self, f: impl Sized) -> Result<DataPayload<M2>, E>
        where
            M2: DataMarker,
        {
            loop {}
        }
    }
    pub struct DataResponse<M>
    where
        M: DataMarker,
    {
        pub metadata: DataResponseMetadata,
        pub payload: Option<DataPayload<M>>,
    }
}
mod prelude {
    pub use crate::error::DataError;
    pub use crate::key::DataKey;
    pub use crate::marker::DataMarker;
    pub use crate::response::DataPayload;
    pub use crate::response::DataResponse;
    pub use crate::response::DataResponseMetadata;
    pub use yoke;
}
pub use prelude::*;
