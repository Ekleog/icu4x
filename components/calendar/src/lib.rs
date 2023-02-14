pub mod provider {
    use crate::types::IsoWeekday;
    use icu_provider::{yoke, zerofrom};
    use tinystr::TinyStr16;
    use zerovec::ZeroVec;
    pub struct EraStartDate {
        year: i32,
        month: u8,
        day: u8,
    }
    impl ::core::marker::Copy for EraStartDate { }
    impl ::core::clone::Clone for EraStartDate {
        fn clone(&self) -> EraStartDate {loop{}}
    }
    impl zerovec::ule::AsULE for EraStartDate {
        type ULE = EraStartDateULE;
        fn to_unaligned(self) -> Self::ULE {loop{}}
        fn from_unaligned(unaligned: Self::ULE) -> Self {loop{}}
    }
    pub struct EraStartDateULE {
        year: <i32 as zerovec::ule::AsULE>::ULE,
        month: <u8 as zerovec::ule::AsULE>::ULE,
        day: <u8 as zerovec::ule::AsULE>::ULE,
    }
    impl ::core::marker::Copy for EraStartDateULE { }
    impl ::core::clone::Clone for EraStartDateULE {
        fn clone(&self) -> EraStartDateULE {loop{}}
    }
    unsafe impl zerovec::ule::ULE for EraStartDateULE {
        fn validate_byte_slice(bytes: &[u8])
            -> Result<(), zerovec::ZeroVecError> {loop{}}
    }
    pub struct JapaneseErasV1Marker;
    impl icu_provider::DataMarker for JapaneseErasV1Marker {
        type Yokeable = JapaneseErasV1<'static>;
    }
    impl icu_provider::KeyedDataMarker for JapaneseErasV1Marker {
        const KEY: icu_provider::DataKey =
            {
                const RESOURCE_KEY_MACRO_CONST: ::icu_provider::DataKey =
                    {
                        match ::icu_provider::DataKey::construct_internal("\nicu4x_key_tagcalendar/japanese@1\n",
                                icu_provider::DataKeyMetadata::construct_internal(icu_provider::FallbackPriority::const_default(),
                                    None, None)) {
                            Ok(v) =>
                                v,
                                Err(_) =>
                                {loop{}},
                        }
                    };
                RESOURCE_KEY_MACRO_CONST
            };
    }
    pub struct JapaneseExtendedErasV1Marker;
    impl icu_provider::DataMarker for JapaneseExtendedErasV1Marker {
        type Yokeable = JapaneseErasV1<'static>;
    }
    impl icu_provider::KeyedDataMarker for JapaneseExtendedErasV1Marker {
        const KEY: icu_provider::DataKey =
            {
                const RESOURCE_KEY_MACRO_CONST: ::icu_provider::DataKey =
                    {
                        match ::icu_provider::DataKey::construct_internal("\nicu4x_key_tagcalendar/japanext@1\n",
                                icu_provider::DataKeyMetadata::construct_internal(icu_provider::FallbackPriority::const_default(),
                                    None, None)) {
                            Ok(v) =>
                                v,
                                Err(_) =>
                                {loop{}},
                        }
                    };
                RESOURCE_KEY_MACRO_CONST
            };
    }
    pub struct JapaneseErasV1<'data> {
        pub dates_to_eras: ZeroVec<'data, (EraStartDate, TinyStr16)>,
    }
    unsafe impl<'a> yoke::Yokeable<'a> for JapaneseErasV1<'static>   {
        type Output = JapaneseErasV1<'a>;
        fn transform(& self) -> & Self::Output {loop{}}
        fn transform_owned(self) -> Self::Output {loop{}}
        unsafe fn make(this: Self::Output) -> Self {loop{}}
        fn transform_mut<F>(& mut self, f: F) where F: 'static
             {loop{}}
    }
    impl<'zf, 'zf_inner> zerofrom::ZeroFrom<'zf, JapaneseErasV1<'zf_inner>>
        for JapaneseErasV1<'zf>   {
        fn zero_from(this: & JapaneseErasV1) -> Self {loop{}}
    }
    pub struct WeekDataV1Marker;
    impl icu_provider::DataMarker for WeekDataV1Marker {
        type Yokeable = WeekDataV1;
    }
    impl icu_provider::KeyedDataMarker for WeekDataV1Marker {
        const KEY: icu_provider::DataKey =
            {
                const RESOURCE_KEY_MACRO_CONST: ::icu_provider::DataKey =
                    {
                        match ::icu_provider::DataKey::construct_internal("\nicu4x_key_tagdatetime/week_data@1\n",
                                icu_provider::DataKeyMetadata::construct_internal(icu_provider::FallbackPriority::Region,
                                    None, None)) {
                            Ok(v) =>
                                v,
                                Err(_) =>
                                {loop{}},
                        }
                    };
                RESOURCE_KEY_MACRO_CONST
            };
    }
    pub struct WeekDataV1 {
        pub first_weekday: IsoWeekday,
        pub min_week_days: u8,
    }
    unsafe impl<'a> yoke::Yokeable<'a> for WeekDataV1   {
        type Output = Self;
        fn transform(&self) -> &Self::Output {loop{}}
        fn transform_owned(self) -> Self::Output {loop{}}
        unsafe fn make(this: Self::Output) -> Self {loop{}}
        fn transform_mut<F>(& mut self, f: F) where F: 'static +
            for<'b> FnOnce(& mut Self::Output) {loop{}}
    }
    impl<'zf> zerofrom::ZeroFrom<'zf, WeekDataV1> for WeekDataV1   {
        fn zero_from(this: & Self) -> Self {loop{}}
    }
}
pub mod types {
    macro_rules! dt_unit {
        (,,, : expr) =>
        {
            # # pub struct ; impl impl for { type = ; fn from_str -> <, :: > }
            impl < > for impl < > for impl
        } ;
    }
    pub enum IsoWeekday {
        Monday = 1,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
        Sunday,
    }
}
