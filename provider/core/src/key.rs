use core::ops::Deref;
use writeable::{LengthHint, Writeable};
use zerovec::ule::*;
macro_rules! tagged {
    ($without_tags:expr) => {
        concat!(
            $crate::leading_tag!(),
            $without_tags,
            $crate::trailing_tag!()
        )
    };
}

#[repr(transparent)]
 struct DataKeyHash([u8; 4]);
 enum FallbackPriority {
    Language,
    Region,
    Collation,
}
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, )]
 enum FallbackSupplement {
    Collation,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, )]
 struct DataKeyPath {
    tagged: &'static str,
}
impl DataKeyPath {
     const fn get(self) -> &'static str {loop{}}
}
impl Deref for DataKeyPath {
    type Target = str;
    fn deref(&self) -> &Self::Target {loop{}}
}
 struct DataKeyMetadata {
     fallback_priority: FallbackPriority,
     extension_key: Option<icu_locid::extensions::unicode::Key>,
     fallback_supplement: Option<FallbackSupplement>,
}
impl Default for DataKeyMetadata {
    #[inline]
    fn default() -> Self {loop{}}
}
pub struct DataKey {
    path: DataKeyPath,
    hash: DataKeyHash,
    metadata: DataKeyMetadata,
}
