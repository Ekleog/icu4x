use core::fmt;
use icu_locid::extensions::unicode as unicode_ext;
use icu_locid::{LanguageIdentifier, Locale, SubtagOrderingResult};
#[derive(PartialEq, Clone, Default, Eq, Hash)]
pub struct DataLocale {
    langid: LanguageIdentifier,
    keywords: unicode_ext::Keywords,
}
impl fmt::Debug for DataLocale {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {loop{}}
}
