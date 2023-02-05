#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

mod date {

    use crate::any_calendar::{AnyCalendar, IntoAnyCalendar};
    use crate::week::{WeekCalculator, WeekOf};
    use crate::{types, Calendar, CalendarError, DateDuration, DateDurationUnit, Iso};
    use alloc::rc::Rc;
    use alloc::sync::Arc;
    use core::fmt;
    use core::ops::Deref;

    pub trait AsCalendar {
        type Calendar: Calendar;
        fn as_calendar(&self) -> &Self::Calendar;
    }

    impl<C: Calendar> AsCalendar for C {
        type Calendar = C;
        #[inline]
        fn as_calendar(&self) -> &Self {
            self
        }
    }

    impl<C: Calendar> AsCalendar for Rc<C> {
        type Calendar = C;
        #[inline]
        fn as_calendar(&self) -> &C {
            self
        }
    }

    impl<C: Calendar> AsCalendar for Arc<C> {
        type Calendar = C;
        #[inline]
        fn as_calendar(&self) -> &C {
            self
        }
    }

    #[allow(clippy::exhaustive_structs)] // newtype
    #[derive(PartialEq, Eq, Debug)]
    pub struct Ref<'a, C>(pub &'a C);

    impl<C> Copy for Ref<'_, C> {}

    impl<C> Clone for Ref<'_, C> {
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<C: Calendar> AsCalendar for Ref<'_, C> {
        type Calendar = C;
        #[inline]
        fn as_calendar(&self) -> &C {
            self.0
        }
    }

    impl<'a, C> Deref for Ref<'a, C> {
        type Target = C;
        fn deref(&self) -> &C {
            self.0
        }
    }

    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    pub struct Date<A: AsCalendar> {
        pub(crate) inner: <A::Calendar as Calendar>::DateInner,
        pub(crate) calendar: A,
    }

    impl<A: AsCalendar> Date<A> {
        #[inline]
        pub fn try_new_from_codes(
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
            calendar: A,
        ) -> Result<Self, CalendarError> {
            let inner = calendar
                .as_calendar()
                .date_from_codes(era, year, month_code, day)?;
            Ok(Date { inner, calendar })
        }

        #[inline]
        pub fn new_from_iso(iso: Date<Iso>, calendar: A) -> Self {
            let inner = calendar.as_calendar().date_from_iso(iso);
            Date { inner, calendar }
        }

        #[inline]
        pub fn to_iso(&self) -> Date<Iso> {
            self.calendar.as_calendar().date_to_iso(self.inner())
        }

        #[inline]
        pub fn to_calendar<A2: AsCalendar>(&self, calendar: A2) -> Date<A2> {
            Date::new_from_iso(self.to_iso(), calendar)
        }

        #[inline]
        pub fn months_in_year(&self) -> u8 {
            self.calendar.as_calendar().months_in_year(self.inner())
        }

        #[inline]
        pub fn days_in_year(&self) -> u32 {
            self.calendar.as_calendar().days_in_year(self.inner())
        }

        #[inline]
        pub fn days_in_month(&self) -> u8 {
            self.calendar.as_calendar().days_in_month(self.inner())
        }

        #[inline]
        pub fn day_of_week(&self) -> types::IsoWeekday {
            self.calendar.as_calendar().day_of_week(self.inner())
        }

        #[doc(hidden)]
        #[inline]
        pub fn add(&mut self, duration: DateDuration<A::Calendar>) {
            self.calendar
                .as_calendar()
                .offset_date(&mut self.inner, duration)
        }

        #[doc(hidden)]
        #[inline]
        pub fn added(mut self, duration: DateDuration<A::Calendar>) -> Self {
            self.add(duration);
            self
        }

        #[doc(hidden)]
        #[inline]
        pub fn until<B: AsCalendar<Calendar = A::Calendar>>(
            &self,
            other: &Date<B>,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<A::Calendar> {
            self.calendar.as_calendar().until(
                self.inner(),
                other.inner(),
                other.calendar.as_calendar(),
                largest_unit,
                smallest_unit,
            )
        }

        #[inline]
        pub fn year(&self) -> types::FormattableYear {
            self.calendar.as_calendar().year(&self.inner)
        }

        #[inline]
        pub fn month(&self) -> types::FormattableMonth {
            self.calendar.as_calendar().month(&self.inner)
        }

        #[inline]
        pub fn day_of_month(&self) -> types::DayOfMonth {
            self.calendar.as_calendar().day_of_month(&self.inner)
        }

        #[inline]
        pub fn day_of_year_info(&self) -> types::DayOfYearInfo {
            self.calendar.as_calendar().day_of_year_info(&self.inner)
        }

        pub fn week_of_month(&self, first_weekday: types::IsoWeekday) -> types::WeekOfMonth {
            let config = WeekCalculator {
                first_weekday,
                min_week_days: 0, // ignored
            };
            config.week_of_month(self.day_of_month(), self.day_of_week())
        }

        pub fn week_of_year(&self, config: &WeekCalculator) -> Result<WeekOf, CalendarError> {
            config.week_of_year(self.day_of_year_info(), self.day_of_week())
        }

        #[inline]
        pub fn from_raw(inner: <A::Calendar as Calendar>::DateInner, calendar: A) -> Self {
            Self { inner, calendar }
        }

        #[inline]
        pub fn inner(&self) -> &<A::Calendar as Calendar>::DateInner {
            &self.inner
        }

        #[inline]
        pub fn calendar(&self) -> &A::Calendar {
            self.calendar.as_calendar()
        }

        #[inline]
        pub fn calendar_wrapper(&self) -> &A {
            &self.calendar
        }
    }

    impl<C: IntoAnyCalendar, A: AsCalendar<Calendar = C>> Date<A> {
        pub fn to_any(&self) -> Date<AnyCalendar> {
            let cal = self.calendar();
            Date::from_raw(cal.date_to_any(self.inner()), cal.to_any_cloned())
        }
    }

    impl<C: Calendar> Date<C> {
        pub fn wrap_calendar_in_rc(self) -> Date<Rc<C>> {
            Date::from_raw(self.inner, Rc::new(self.calendar))
        }

        pub fn wrap_calendar_in_arc(self) -> Date<Arc<C>> {
            Date::from_raw(self.inner, Arc::new(self.calendar))
        }
    }

    impl<C, A, B> PartialEq<Date<B>> for Date<A>
    where
        C: Calendar,
        A: AsCalendar<Calendar = C>,
        B: AsCalendar<Calendar = C>,
    {
        fn eq(&self, other: &Date<B>) -> bool {
            self.inner.eq(&other.inner)
        }
    }

    impl<A: AsCalendar> Eq for Date<A> {}

    impl<A: AsCalendar> fmt::Debug for Date<A> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            write!(
                f,
                "Date({:?}, for calendar {})",
                self.inner,
                self.calendar.as_calendar().debug_name()
            )
        }
    }

    impl<A: AsCalendar + Clone> Clone for Date<A> {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
                calendar: self.calendar.clone(),
            }
        }
    }

    impl<A> Copy for Date<A>
    where
        A: AsCalendar + Copy,
        <<A as AsCalendar>::Calendar as Calendar>::DateInner: Copy,
    {
    }
}
mod datetime {

    use crate::any_calendar::{AnyCalendar, IntoAnyCalendar};
    use crate::types::{self, Time};
    use crate::{AsCalendar, Calendar, CalendarError, Date, Iso};
    use alloc::rc::Rc;
    use alloc::sync::Arc;

    #[derive(Debug)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct DateTime<A: AsCalendar> {
        pub date: Date<A>,
        pub time: Time,
    }

    impl<A: AsCalendar> DateTime<A> {
        pub fn new(date: Date<A>, time: Time) -> Self {
            DateTime { date, time }
        }

        #[inline]
        pub fn try_new_from_codes(
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
            time: Time,
            calendar: A,
        ) -> Result<Self, CalendarError> {loop {}
        }

        #[inline]
        pub fn new_from_iso(iso: DateTime<Iso>, calendar: A) -> Self {loop {}
        }

        #[inline]
        pub fn to_iso(&self) -> DateTime<Iso> {loop {}
        }

        #[inline]
        pub fn to_calendar<A2: AsCalendar>(&self, calendar: A2) -> DateTime<A2> {loop {}
        }
    }

    impl<C: IntoAnyCalendar, A: AsCalendar<Calendar = C>> DateTime<A> {
        pub fn to_any(&self) -> DateTime<AnyCalendar> {loop {}
        }
    }

    impl<C: Calendar> DateTime<C> {
        pub fn wrap_calendar_in_rc(self) -> DateTime<Rc<C>> {loop {}
        }

        pub fn wrap_calendar_in_arc(self) -> DateTime<Arc<C>> {loop {}
        }
    }

    impl<C, A, B> PartialEq<DateTime<B>> for DateTime<A>
    where
        C: Calendar,
        A: AsCalendar<Calendar = C>,
        B: AsCalendar<Calendar = C>,
    {
        fn eq(&self, other: &DateTime<B>) -> bool {loop {}
        }
    }

    impl<A: AsCalendar> Eq for DateTime<A> {}

    impl<A: AsCalendar + Clone> Clone for DateTime<A> {
        fn clone(&self) -> Self {loop {}
        }
    }

    impl<A> Copy for DateTime<A>
    where
        A: AsCalendar + Copy,
        <<A as AsCalendar>::Calendar as Calendar>::DateInner: Copy,
    {
    }
}

pub mod any_calendar {


    use crate::buddhist::Buddhist;
    use crate::coptic::Coptic;
    use crate::ethiopian::{Ethiopian, EthiopianEraStyle};
    use crate::gregorian::Gregorian;
    use crate::indian::Indian;
    use crate::iso::Iso;
    use crate::japanese::{Japanese, JapaneseExtended};
    use crate::{
        types, AsCalendar, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime,
        Ref,
    };

    use icu_locid::{
        extensions::unicode::Value, extensions_unicode_key as key,
        extensions_unicode_value as value, subtags_language as language, Locale,
    };
    use icu_provider::prelude::*;

    use core::fmt;

    #[non_exhaustive]
    #[derive(Clone, Debug)]
    #[cfg_attr(all(test, feature = "serde"), derive(bolero::generator::TypeGenerator))]
    pub enum AnyCalendar {
        Gregorian(Gregorian),
        Buddhist(Buddhist),
        Japanese(
            #[cfg_attr(
            all(test, feature = "serde"),
            generator(bolero::generator::constant(Japanese::try_new_unstable(
                &icu_testdata::buffer().as_deserializing()
            ).unwrap()))
        )]
            Japanese,
        ),
        JapaneseExtended(
            #[cfg_attr(
            all(test, feature = "serde"),
            generator(bolero::generator::constant(JapaneseExtended::try_new_unstable(
                &icu_testdata::buffer().as_deserializing()
            ).unwrap()))
        )]
            JapaneseExtended,
        ),
        Ethiopian(Ethiopian),
        Indian(Indian),
        Coptic(Coptic),
        Iso(Iso),
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    #[non_exhaustive]
    pub enum AnyDateInner {
        Gregorian(<Gregorian as Calendar>::DateInner),
        Buddhist(<Buddhist as Calendar>::DateInner),
        Japanese(<Japanese as Calendar>::DateInner),
        JapaneseExtended(<JapaneseExtended as Calendar>::DateInner),
        Ethiopian(<Ethiopian as Calendar>::DateInner),
        Indian(<Indian as Calendar>::DateInner),
        Coptic(<Coptic as Calendar>::DateInner),
        Iso(<Iso as Calendar>::DateInner),
    }

    macro_rules! match_cal_and_date {
        (match ($cal:ident, $date:ident): ($cal_matched:ident, $date_matched:ident) => $e:expr) => {
            loop {}
        };
    }

    impl Calendar for AnyCalendar {
        type DateInner = AnyDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            loop {}
        }
        fn date_from_iso(&self, iso: Date<Iso>) -> AnyDateInner {loop {}
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {loop {}
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {loop {}
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {loop {}
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {loop {}
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            loop {}
        }

        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> { loop {} }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear { loop {} }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth { loop {} }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth { loop {} }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo { loop {} }

        fn debug_name(&self) -> &'static str { loop {} }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> { loop {} }
    }

    impl AnyCalendar {
        pub fn try_new_with_any_provider<P>(
            provider: &P,
            kind: AnyCalendarKind,
        ) -> Result<Self, CalendarError>
        where
            P: AnyProvider + ?Sized,
            { loop {} }

        #[cfg(feature = "serde")]
        pub fn try_new_with_buffer_provider<P>(
            provider: &P,
            kind: AnyCalendarKind,
        ) -> Result<Self, CalendarError>
        where
            P: BufferProvider + ?Sized,
            { loop {} }

        pub fn try_new_unstable<P>(
            provider: &P,
            kind: AnyCalendarKind,
        ) -> Result<Self, CalendarError>
        where
            P: DataProvider<crate::provider::JapaneseErasV1Marker>
                + DataProvider<crate::provider::JapaneseExtendedErasV1Marker>
                + ?Sized,
                { loop {} }

        icu_provider::gen_any_buffer_constructors!(
            locale: include,
            options: skip,
            error: CalendarError,
            functions: [
                Self::try_new_for_locale_unstable,
                try_new_for_locale_with_any_provider,
                try_new_for_locale_with_buffer_provider
            ]
        );

        pub fn try_new_for_locale_unstable<P>(
            provider: &P,
            locale: &DataLocale,
        ) -> Result<Self, CalendarError>
        where
            P: DataProvider<crate::provider::JapaneseErasV1Marker>
                + DataProvider<crate::provider::JapaneseExtendedErasV1Marker>
                + ?Sized,
                { loop {} }

        fn calendar_name(&self) -> &'static str { loop {} }

        pub fn kind(&self) -> AnyCalendarKind { loop {} }

        pub fn convert_any_date<'a>(
            &'a self,
            date: &Date<impl AsCalendar<Calendar = AnyCalendar>>,
        ) -> Date<Ref<'a, AnyCalendar>> { loop {} }

        pub fn convert_any_datetime<'a>(
            &'a self,
            date: &DateTime<impl AsCalendar<Calendar = AnyCalendar>>,
        ) -> DateTime<Ref<'a, AnyCalendar>> { loop {} }
    }

    impl AnyDateInner {
        fn calendar_name(&self) -> &'static str { loop {} }
    }

    #[non_exhaustive]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    pub enum AnyCalendarKind {
        Gregorian,
        Buddhist,
        Japanese,
        JapaneseExtended,
        Ethiopian,
        EthiopianAmeteAlem,
        Indian,
        Coptic,
        Iso,
    }

    impl AnyCalendarKind {
        pub fn get_for_bcp47_string(x: &str) -> Option<Self> { loop {} }
        pub fn get_for_bcp47_bytes(x: &[u8]) -> Option<Self> { loop {} }
        pub fn get_for_bcp47_value(x: &Value) -> Option<Self> { loop {} }

        pub fn as_bcp47_string(self) -> &'static str { loop {} }

        pub fn as_bcp47_value(self) -> Value { loop {} }

        pub fn get_for_locale(l: &Locale) -> Option<Self> { loop {} }

        fn get_for_data_locale(l: &DataLocale) -> Option<Self> { loop {} }

        fn from_data_locale_with_fallback(l: &DataLocale) -> Self { loop {} }
    }

    impl fmt::Display for AnyCalendarKind {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Debug::fmt(self, f)
        }
    }

    impl<C: IntoAnyCalendar> From<C> for AnyCalendar {
        fn from(c: C) -> AnyCalendar {
            c.to_any()
        }
    }

    pub trait IntoAnyCalendar: Calendar + Sized {
        fn to_any(self) -> AnyCalendar;

        fn to_any_cloned(&self) -> AnyCalendar;
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner;
    }

    impl IntoAnyCalendar for Gregorian {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Buddhist {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Japanese {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for JapaneseExtended {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Ethiopian {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Indian {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Coptic {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    impl IntoAnyCalendar for Iso {
        fn to_any(self) -> AnyCalendar { loop {} }
        fn to_any_cloned(&self) -> AnyCalendar { loop {} }
        fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner { loop {} }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::Ref;
        use core::convert::TryInto;

        fn single_test_roundtrip(
            calendar: Ref<AnyCalendar>,
            era: &str,
            year: i32,
            month_code: &str,
            day: u8,
        ) { loop {} }

        fn single_test_error(
            calendar: Ref<AnyCalendar>,
            era: &str,
            year: i32,
            month_code: &str,
            day: u8,
            error: CalendarError,
        ) { loop {} }
    }
}
pub mod buddhist {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::ArithmeticDate;
    use crate::iso::{Iso, IsoDateInner};
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use tinystr::tinystr;

    const BUDDHIST_ERA_OFFSET: i32 = 543;

    #[derive(Copy, Clone, Debug, Default)]

    #[allow(clippy::exhaustive_structs)] // this type is stable
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    pub struct Buddhist;

    impl Calendar for Buddhist {
        type DateInner = IsoDateInner;

        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> { loop {} }
        fn date_from_iso(&self, iso: Date<Iso>) -> IsoDateInner { loop {} }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> { loop {} }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 { loop {} }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 { loop {} }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 { loop {} }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) { loop {} }

        #[allow(clippy::field_reassign_with_default)] // it's more clear this way
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> { loop {} }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear { loop {} }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth { loop {} }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth { loop {} }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo { loop {} }

        fn debug_name(&self) -> &'static str { loop {} }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> { loop {} }
    }

    impl Date<Buddhist> {
        pub fn try_new_buddhist_date(
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Buddhist>, CalendarError> { loop {} }
    }

    impl DateTime<Buddhist> {
        pub fn try_new_buddhist_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Buddhist>, CalendarError> { loop {} }
    }

    fn iso_year_as_buddhist(year: i32) -> types::FormattableYear { loop {} }
}
mod calendar {

    use crate::any_calendar::AnyCalendarKind;
    use crate::{types, CalendarError, Date, DateDuration, DateDurationUnit, Iso};
    use core::fmt;

    pub trait Calendar {
        type DateInner: PartialEq + Eq + Clone + fmt::Debug;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError>;
        fn date_from_iso(&self, iso: Date<Iso>) -> Self::DateInner;
        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso>;

        fn months_in_year(&self, date: &Self::DateInner) -> u8;
        fn days_in_year(&self, date: &Self::DateInner) -> u32;
        fn days_in_month(&self, date: &Self::DateInner) -> u8;
        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            self.date_to_iso(date).day_of_week()
        }

        #[doc(hidden)] // unstable
        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>);

        #[doc(hidden)] // unstable
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self>;

        fn debug_name(&self) -> &'static str;

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear;

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth;

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth;

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo;

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            None
        }
    }
}
mod calendar_arithmetic {

    use crate::{types, Calendar, CalendarError, DateDuration, DateDurationUnit};
    use core::convert::TryInto;
    use core::marker::PhantomData;
    use tinystr::tinystr;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct ArithmeticDate<C: CalendarArithmetic> {
        pub year: i32,
        pub month: u8,
        pub day: u8,
        pub marker: PhantomData<C>,
    }

    pub trait CalendarArithmetic: Calendar {
        fn month_days(year: i32, month: u8) -> u8;
        fn months_for_every_year(year: i32) -> u8;
        fn is_leap_year(year: i32) -> bool;

        fn days_in_provided_year(year: i32) -> u32 { loop {} }
    }

    impl<C: CalendarArithmetic> ArithmeticDate<C> {
        #[inline]
        pub fn new(year: i32, month: u8, day: u8) -> Self { loop {} }

        #[inline]
        fn offset_days(&mut self, mut day_offset: i32) { loop {} }

        #[inline]
        fn offset_months(&mut self, mut month_offset: i32) { loop {} }

        #[inline]
        pub fn offset_date(&mut self, offset: DateDuration<C>) { loop {} }

        #[inline]
        pub fn until(
            &self,
            date2: ArithmeticDate<C>,
            _largest_unit: DateDurationUnit,
            _smaller_unti: DateDurationUnit,
        ) -> DateDuration<C> { loop {} }

        #[inline]
        pub fn days_in_year(&self) -> u32 { loop {} }

        #[inline]
        pub fn months_in_year(&self) -> u8 { loop {} }

        #[inline]
        pub fn days_in_month(&self) -> u8 { loop {} }

        #[inline]
        pub fn day_of_year(&self) -> u32 { loop {} }

        #[inline]
        pub fn date_from_year_day(year: i32, year_day: u32) -> ArithmeticDate<C> {
            let mut month = 1;
            let mut day = year_day as i32;
            while month <= C::months_for_every_year(year) {
                let month_days = C::month_days(year, month) as i32;
                if day <= month_days {
                    break;
                } else {
                    day -= month_days;
                    month += 1;
                }
            }

            debug_assert!(day <= C::month_days(year, month) as i32);
            #[allow(clippy::unwrap_used)]
            ArithmeticDate {
                year,
                month,
                day: day.try_into().unwrap_or(0),
                marker: PhantomData,
            }
        }

        #[inline]
        pub fn day_of_month(&self) -> types::DayOfMonth {
            types::DayOfMonth(self.day.into())
        }

        #[inline]
        pub fn solar_month(&self) -> types::FormattableMonth {
            let code = match self.month {
                a if a > C::months_for_every_year(self.year) => tinystr!(4, "und"),
                1 => tinystr!(4, "M01"),
                2 => tinystr!(4, "M02"),
                3 => tinystr!(4, "M03"),
                4 => tinystr!(4, "M04"),
                5 => tinystr!(4, "M05"),
                6 => tinystr!(4, "M06"),
                7 => tinystr!(4, "M07"),
                8 => tinystr!(4, "M08"),
                9 => tinystr!(4, "M09"),
                10 => tinystr!(4, "M10"),
                11 => tinystr!(4, "M11"),
                12 => tinystr!(4, "M12"),
                13 => tinystr!(4, "M13"),
                _ => tinystr!(4, "und"),
            };
            types::FormattableMonth {
                ordinal: self.month as u32,
                code: types::MonthCode(code),
            }
        }

        pub fn new_from_solar<C2: Calendar>(
            cal: &C2,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self, CalendarError> {
            let month = if let Some(ordinal) = ordinal_solar_month_from_code(month_code) {
                ordinal
            } else {
                return Err(CalendarError::UnknownMonthCode(
                    month_code.0,
                    cal.debug_name(),
                ));
            };

            if month > C::months_for_every_year(year) {
                return Err(CalendarError::UnknownMonthCode(
                    month_code.0,
                    cal.debug_name(),
                ));
            }

            if day > C::month_days(year, month) {
                return Err(CalendarError::OutOfRange);
            }

            Ok(Self::new(year, month, day))
        }
    }

    pub fn ordinal_solar_month_from_code(code: types::MonthCode) -> Option<u8> {
        if code.0.len() != 3 {
            return None;
        }
        let bytes = code.0.all_bytes();
        if bytes[0] != b'M' {
            return None;
        }
        if bytes[1] == b'0' {
            if bytes[2] >= b'1' && bytes[2] <= b'9' {
                return Some(bytes[2] - b'0');
            }
        } else if bytes[1] == b'1' && bytes[2] >= b'1' && bytes[2] <= b'3' {
            return Some(10 + bytes[2] - b'0');
        }
        None
    }
}
pub mod coptic {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::quotient;
    use crate::iso::Iso;
    use crate::julian::Julian;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;

    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Coptic;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct CopticDateInner(pub(crate) ArithmeticDate<Coptic>);

    impl CalendarArithmetic for Coptic {
        fn month_days(year: i32, month: u8) -> u8 {
            if (1..=12).contains(&month) {
                30
            } else if month == 13 {
                if Self::is_leap_year(year) {
                    6
                } else {
                    5
                }
            } else {
                0
            }
        }

        fn months_for_every_year(_: i32) -> u8 {
            13
        }

        fn is_leap_year(year: i32) -> bool {
            year % 4 == 3
        }

        fn days_in_provided_year(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Calendar for Coptic {
        type DateInner = CopticDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            let year = if era.0 == tinystr!(16, "ad") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                year
            } else if era.0 == tinystr!(16, "bd") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                1 - year
            } else {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            };

            ArithmeticDate::new_from_solar(self, year, month_code, day).map(CopticDateInner)
        }
        fn date_from_iso(&self, iso: Date<Iso>) -> CopticDateInner {
            let fixed_iso = Iso::fixed_from_iso(*iso.inner());
            Self::coptic_from_fixed(fixed_iso)
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            let fixed_coptic = Coptic::fixed_from_coptic(date.0);
            Iso::iso_from_fixed(fixed_coptic)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            date.0.months_in_year()
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            date.0.days_in_year()
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            date.0.days_in_month()
        }

        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            Iso.day_of_week(Coptic.date_to_iso(date).inner())
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            date.0.offset_date(offset);
        }

        #[allow(clippy::field_reassign_with_default)]
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            _largest_unit: DateDurationUnit,
            _smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            date1.0.until(date2.0, _largest_unit, _smallest_unit)
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            year_as_coptic(date.0.year)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            date.0.day_of_month()
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = date.0.year - 1;
            let next_year = date.0.year + 1;
            types::DayOfYearInfo {
                day_of_year: date.0.day_of_year(),
                days_in_year: date.0.days_in_year(),
                prev_year: year_as_coptic(prev_year),
                days_in_prev_year: Coptic::days_in_year_direct(prev_year),
                next_year: year_as_coptic(next_year),
            }
        }

        fn debug_name(&self) -> &'static str {
            "Coptic"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::Coptic)
        }
    }

    pub(crate) const COPTIC_EPOCH: i32 = Julian::fixed_from_julian_integers(284, 8, 29);

    impl Coptic {
        fn fixed_from_coptic(date: ArithmeticDate<Coptic>) -> i32 {
            COPTIC_EPOCH - 1
                + 365 * (date.year - 1)
                + quotient(date.year, 4)
                + 30 * (date.month as i32 - 1)
                + date.day as i32
        }

        pub(crate) fn fixed_from_coptic_integers(year: i32, month: u8, day: u8) -> i32 {
            Self::fixed_from_coptic(ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            })
        }

        pub(crate) fn coptic_from_fixed(date: i32) -> CopticDateInner {
            let year = quotient(4 * (date - COPTIC_EPOCH) + 1463, 1461);
            let month =
                (quotient(date - Self::fixed_from_coptic_integers(year, 1, 1), 30) + 1) as u8; // <= 12 < u8::MAX
            let day = (date + 1 - Self::fixed_from_coptic_integers(year, month, 1)) as u8; // <= days_in_month < u8::MAX

            #[allow(clippy::unwrap_used)] // day and month have the correct bounds
            *Date::try_new_coptic_date(year, month, day).unwrap().inner()
        }

        fn days_in_year_direct(year: i32) -> u32 {
            if Coptic::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Date<Coptic> {
        pub fn try_new_coptic_date(
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Coptic>, CalendarError> {
            let inner = ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            };

            let bound = inner.days_in_month();
            if day > bound {
                return Err(CalendarError::OutOfRange);
            }

            Ok(Date::from_raw(CopticDateInner(inner), Coptic))
        }
    }

    impl DateTime<Coptic> {
        pub fn try_new_coptic_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Coptic>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_coptic_date(year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    fn year_as_coptic(year: i32) -> types::FormattableYear {
        if year > 0 {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "ad")),
                number: year,
                related_iso: None,
            }
        } else {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "bd")),
                number: 1 - year,
                related_iso: None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_coptic_regression() {
            let iso_date = Date::try_new_iso_date(-100, 3, 3).unwrap();
            let coptic = iso_date.to_calendar(Coptic);
            let recovered_iso = coptic.to_iso();
            assert_eq!(iso_date, recovered_iso);
        }
    }
}
mod duration {

    use crate::Calendar;
    use core::fmt;
    use core::marker::PhantomData;

    #[derive(Copy, Clone, Eq, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type should be stable (and is intended to be constructed manually)
    pub struct DateDuration<C: Calendar + ?Sized> {
        pub years: i32,
        pub months: i32,
        pub weeks: i32,
        pub days: i32,
        pub marker: PhantomData<C>,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    #[allow(clippy::exhaustive_enums)] // this type should be stable
    pub enum DateDurationUnit {
        Years,
        Months,
        Weeks,
        Days,
    }

    impl<C: Calendar + ?Sized> Default for DateDuration<C> {
        fn default() -> Self {
            Self {
                years: 0,
                months: 0,
                weeks: 0,
                days: 0,
                marker: PhantomData,
            }
        }
    }

    impl<C: Calendar + ?Sized> DateDuration<C> {
        pub fn new(years: i32, months: i32, weeks: i32, days: i32) -> Self {
            DateDuration {
                years,
                months,
                weeks,
                days,
                marker: PhantomData,
            }
        }

        pub fn cast_unit<C2: Calendar + ?Sized>(self) -> DateDuration<C2> {
            DateDuration {
                years: self.years,
                months: self.months,
                days: self.days,
                weeks: self.weeks,
                marker: PhantomData,
            }
        }
    }

    impl<C: Calendar> fmt::Debug for DateDuration<C> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
            f.debug_struct("DateDuration")
                .field("years", &self.years)
                .field("months", &self.months)
                .field("weeks", &self.weeks)
                .field("days", &self.days)
                .finish()
        }
    }
}
mod error {

    use displaydoc::Display;
    use icu_provider::DataError;
    use tinystr::{tinystr, TinyStr16, TinyStr4};
    use writeable::Writeable;

    #[cfg(feature = "std")]
    impl std::error::Error for CalendarError {}

    #[derive(Display, Debug, Copy, Clone, PartialEq)]
    #[non_exhaustive]
    pub enum CalendarError {
        #[displaydoc("Could not parse as integer")]
        Parse,
        #[displaydoc("{field} must be between 0-{max}")]
        Overflow {
            field: &'static str,
            max: usize,
        },
        #[displaydoc("{field} must be between {min}-0")]
        Underflow {
            field: &'static str,
            min: isize,
        },
        /// Foo
        OutOfRange,
        #[displaydoc("No era named {0} for calendar {1}")]
        UnknownEra(TinyStr16, &'static str),
        #[displaydoc("No month code named {0} for calendar {1}")]
        UnknownMonthCode(TinyStr4, &'static str),
        #[displaydoc("No value for {0}")]
        MissingInput(&'static str),
        #[displaydoc("AnyCalendar does not support calendar {0}")]
        UnknownAnyCalendarKind(TinyStr16),
        #[displaydoc("An operation required a calendar but a calendar was not provided")]
        MissingCalendar,
        #[displaydoc("{0}")]
        Data(DataError),
    }

    impl From<core::num::ParseIntError> for CalendarError {
        fn from(_: core::num::ParseIntError) -> Self {
            CalendarError::Parse
        }
    }

    impl From<DataError> for CalendarError {
        fn from(e: DataError) -> Self {
            CalendarError::Data(e)
        }
    }

    impl CalendarError {
        pub fn unknown_any_calendar_kind(description: impl Writeable) -> Self {
            let tiny = description
                .write_to_string()
                .get(0..16)
                .and_then(|x| TinyStr16::from_str(x).ok())
                .unwrap_or(tinystr!(16, "invalid"));
            Self::UnknownAnyCalendarKind(tiny)
        }
    }
}
pub mod ethiopian {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::coptic::Coptic;
    use crate::iso::Iso;
    use crate::julian::Julian;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;

    const AMETE_ALEM_OFFSET: i32 = 5500;

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    #[non_exhaustive]
    pub enum EthiopianEraStyle {
        AmeteMihret,
        AmeteAlem,
    }

    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    pub struct Ethiopian(pub(crate) bool);

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct EthiopianDateInner(ArithmeticDate<Ethiopian>);

    impl CalendarArithmetic for Ethiopian {
        fn month_days(year: i32, month: u8) -> u8 {
            if (1..=12).contains(&month) {
                30
            } else if month == 13 {
                if Self::is_leap_year(year) {
                    6
                } else {
                    5
                }
            } else {
                0
            }
        }

        fn months_for_every_year(_: i32) -> u8 {
            13
        }

        fn is_leap_year(year: i32) -> bool {
            year % 4 == 3
        }

        fn days_in_provided_year(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Calendar for Ethiopian {
        type DateInner = EthiopianDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            let year = if era.0 == tinystr!(16, "incar") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                year
            } else if era.0 == tinystr!(16, "pre-incar") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                1 - year
            } else if era.0 == tinystr!(16, "mundi") {
                year - AMETE_ALEM_OFFSET
            } else {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            };

            ArithmeticDate::new_from_solar(self, year, month_code, day).map(EthiopianDateInner)
        }
        fn date_from_iso(&self, iso: Date<Iso>) -> EthiopianDateInner {
            let fixed_iso = Iso::fixed_from_iso(*iso.inner());
            Self::ethiopian_from_fixed(fixed_iso)
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            let fixed_ethiopian = Ethiopian::fixed_from_ethiopian(date.0);
            Iso::iso_from_fixed(fixed_ethiopian)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            date.0.months_in_year()
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            date.0.days_in_year()
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            date.0.days_in_month()
        }

        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            Iso.day_of_week(self.date_to_iso(date).inner())
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            date.0.offset_date(offset);
        }

        #[allow(clippy::field_reassign_with_default)]
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            _largest_unit: DateDurationUnit,
            _smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            date1.0.until(date2.0, _largest_unit, _smallest_unit)
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            Self::year_as_ethiopian(date.0.year, self.0)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            date.0.day_of_month()
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = date.0.year - 1;
            let next_year = date.0.year + 1;
            types::DayOfYearInfo {
                day_of_year: date.0.day_of_year(),
                days_in_year: date.0.days_in_year(),
                prev_year: Self::year_as_ethiopian(prev_year, self.0),
                days_in_prev_year: Ethiopian::days_in_year_direct(prev_year),
                next_year: Self::year_as_ethiopian(next_year, self.0),
            }
        }

        fn debug_name(&self) -> &'static str {
            "Ethiopian"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            if self.0 {
                Some(AnyCalendarKind::EthiopianAmeteAlem)
            } else {
                Some(AnyCalendarKind::Ethiopian)
            }
        }
    }

    const ETHIOPIC_TO_COPTIC_OFFSET: i32 =
        super::coptic::COPTIC_EPOCH - Julian::fixed_from_julian_integers(8, 8, 29);

    impl Ethiopian {
        pub fn new() -> Self {
            Self(false)
        }
        pub fn new_with_era_style(era_style: EthiopianEraStyle) -> Self {
            Self(era_style == EthiopianEraStyle::AmeteAlem)
        }
        pub fn set_era_style(&mut self, era_style: EthiopianEraStyle) {
            self.0 = era_style == EthiopianEraStyle::AmeteAlem
        }

        pub fn era_style(&self) -> EthiopianEraStyle {
            if self.0 {
                EthiopianEraStyle::AmeteAlem
            } else {
                EthiopianEraStyle::AmeteMihret
            }
        }

        fn fixed_from_ethiopian(date: ArithmeticDate<Ethiopian>) -> i32 {
            Coptic::fixed_from_coptic_integers(date.year, date.month, date.day)
                - ETHIOPIC_TO_COPTIC_OFFSET
        }

        fn ethiopian_from_fixed(date: i32) -> EthiopianDateInner {
            let coptic_date = Coptic::coptic_from_fixed(date + ETHIOPIC_TO_COPTIC_OFFSET);

            #[allow(clippy::unwrap_used)]
            *Date::try_new_ethiopian_date(
                EthiopianEraStyle::AmeteMihret,
                coptic_date.0.year,
                coptic_date.0.month,
                coptic_date.0.day,
            )
            .unwrap()
            .inner()
        }

        fn days_in_year_direct(year: i32) -> u32 {
            if Ethiopian::is_leap_year(year) {
                366
            } else {
                365
            }
        }

        fn year_as_ethiopian(year: i32, amete_alem: bool) -> types::FormattableYear {
            if amete_alem {
                types::FormattableYear {
                    era: types::Era(tinystr!(16, "mundi")),
                    number: year + AMETE_ALEM_OFFSET,
                    related_iso: None,
                }
            } else if year > 0 {
                types::FormattableYear {
                    era: types::Era(tinystr!(16, "incar")),
                    number: year,
                    related_iso: None,
                }
            } else {
                types::FormattableYear {
                    era: types::Era(tinystr!(16, "pre-incar")),
                    number: 1 - year,
                    related_iso: None,
                }
            }
        }
    }

    impl Date<Ethiopian> {
        pub fn try_new_ethiopian_date(
            era_style: EthiopianEraStyle,
            mut year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Ethiopian>, CalendarError> {
            if era_style == EthiopianEraStyle::AmeteAlem {
                year -= AMETE_ALEM_OFFSET;
            }
            let inner = ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            };

            let bound = inner.days_in_month();
            if day > bound {
                return Err(CalendarError::OutOfRange);
            }

            Ok(Date::from_raw(
                EthiopianDateInner(inner),
                Ethiopian::new_with_era_style(era_style),
            ))
        }
    }

    impl DateTime<Ethiopian> {
        pub fn try_new_ethiopian_datetime(
            era_style: EthiopianEraStyle,
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Ethiopian>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_ethiopian_date(era_style, year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_leap_year() {
            let iso_date = Date::try_new_iso_date(2023, 9, 11).unwrap();
            let ethiopian_date = Ethiopian::new().date_from_iso(iso_date);
            assert_eq!(ethiopian_date.0.year, 2015);
            assert_eq!(ethiopian_date.0.month, 13);
            assert_eq!(ethiopian_date.0.day, 6);
        }

        #[test]
        fn test_iso_to_ethiopian_conversion_and_back() {
            let iso_date = Date::try_new_iso_date(1970, 1, 2).unwrap();
            let date_ethiopian = Date::new_from_iso(iso_date, Ethiopian::new());

            assert_eq!(date_ethiopian.inner.0.year, 1962);
            assert_eq!(date_ethiopian.inner.0.month, 4);
            assert_eq!(date_ethiopian.inner.0.day, 24);

            assert_eq!(
                date_ethiopian.to_iso(),
                Date::try_new_iso_date(1970, 1, 2).unwrap()
            );
        }

        #[test]
        fn test_roundtrip_negative() {
            let iso_date = Date::try_new_iso_date(-1000, 3, 3).unwrap();
            let ethiopian = iso_date.to_calendar(Ethiopian::new());
            let recovered_iso = ethiopian.to_iso();
            assert_eq!(iso_date, recovered_iso);
        }
    }
}
mod fuzz {
    #![cfg(all(test, feature = "serde"))]

    use crate::{AnyCalendar, DateTime};

    use bolero::generator::gen;

    #[test]
    fn calendar_conversions_round_trip() {
        bolero::check!()
            .with_generator((
                gen::<AnyCalendar>(),
                gen::<AnyCalendar>(),
                -8000..8000,
                1..12,
                1..31,
                0..25,
                0..61,
                0..71,
            ))
            .cloned()
            .for_each(|(from, to, year, month, day, hour, minute, second)| {
                let time =
                    match DateTime::try_new_iso_datetime(year, month, day, hour, minute, second) {
                        Ok(time) => time.to_calendar(from.clone()),
                        Err(_) => return,
                    };
                let converted = time.to_calendar(to);
                let back = converted.to_calendar(from);
                assert_eq!(time, back);
            })
    }
}
pub mod gregorian {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::ArithmeticDate;
    use crate::iso::{Iso, IsoDateInner};
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use tinystr::tinystr;

    #[derive(Copy, Clone, Debug, Default)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Gregorian;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct GregorianDateInner(IsoDateInner);

    impl Calendar for Gregorian {
        type DateInner = GregorianDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            let year = if era.0 == tinystr!(16, "ce") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                year
            } else if era.0 == tinystr!(16, "bce") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                1 - year
            } else {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            };

            ArithmeticDate::new_from_solar(self, year, month_code, day)
                .map(IsoDateInner)
                .map(GregorianDateInner)
        }

        fn date_from_iso(&self, iso: Date<Iso>) -> GregorianDateInner {
            GregorianDateInner(*iso.inner())
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            Date::from_raw(date.0, Iso)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            Iso.months_in_year(&date.0)
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            Iso.days_in_year(&date.0)
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            Iso.days_in_month(&date.0)
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            Iso.offset_date(&mut date.0, offset.cast_unit())
        }

        #[allow(clippy::field_reassign_with_default)] // it's more clear this way
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            Iso.until(&date1.0, &date2.0, &Iso, largest_unit, smallest_unit)
                .cast_unit()
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            year_as_gregorian(date.0 .0.year)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            Iso.month(&date.0)
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            Iso.day_of_month(&date.0)
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = date.0 .0.year - 1;
            let next_year = date.0 .0.year + 1;
            types::DayOfYearInfo {
                day_of_year: Iso::day_of_year(date.0),
                days_in_year: Iso::days_in_year_direct(date.0 .0.year),
                prev_year: year_as_gregorian(prev_year),
                days_in_prev_year: Iso::days_in_year_direct(prev_year),
                next_year: year_as_gregorian(next_year),
            }
        }

        fn debug_name(&self) -> &'static str {
            "Gregorian"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::Gregorian)
        }
    }

    impl Date<Gregorian> {
        pub fn try_new_gregorian_date(
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Gregorian>, CalendarError> {
            Date::try_new_iso_date(year, month, day).map(|d| Date::new_from_iso(d, Gregorian))
        }
    }

    impl DateTime<Gregorian> {
        pub fn try_new_gregorian_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Gregorian>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_gregorian_date(year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    pub(crate) fn year_as_gregorian(year: i32) -> types::FormattableYear {
        if year > 0 {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "ce")),
                number: year,
                related_iso: None,
            }
        } else {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "bce")),
                number: 1 - year,
                related_iso: None,
            }
        }
    }
}
mod helpers {

    pub fn div_rem_euclid(n: i32, d: i32) -> (i32, i32) {
        debug_assert!(d > 0);
        let (a, b) = (n / d, n % d);
        if n >= 0 || b == 0 {
            (a, b)
        } else {
            (a - 1, d + b)
        }
    }

    pub const fn quotient(n: i32, d: i32) -> i32 {
        debug_assert!(d > 0);
        let (a, b) = (n / d, n % d);
        if n >= 0 || b == 0 {
            a
        } else {
            a - 1
        }
    }

    #[test]
    fn test_div_rem_euclid() {
        assert_eq!(div_rem_euclid(i32::MIN, 1), (-2147483648, 0));
        assert_eq!(div_rem_euclid(i32::MIN, 2), (-1073741824, 0));
        assert_eq!(div_rem_euclid(i32::MIN, 3), (-715827883, 1));

        assert_eq!(div_rem_euclid(-10, 1), (-10, 0));
        assert_eq!(div_rem_euclid(-10, 2), (-5, 0));
        assert_eq!(div_rem_euclid(-10, 3), (-4, 2));

        assert_eq!(div_rem_euclid(-9, 1), (-9, 0));
        assert_eq!(div_rem_euclid(-9, 2), (-5, 1));
        assert_eq!(div_rem_euclid(-9, 3), (-3, 0));

        assert_eq!(div_rem_euclid(-8, 1), (-8, 0));
        assert_eq!(div_rem_euclid(-8, 2), (-4, 0));
        assert_eq!(div_rem_euclid(-8, 3), (-3, 1));

        assert_eq!(div_rem_euclid(-2, 1), (-2, 0));
        assert_eq!(div_rem_euclid(-2, 2), (-1, 0));
        assert_eq!(div_rem_euclid(-2, 3), (-1, 1));

        assert_eq!(div_rem_euclid(-1, 1), (-1, 0));
        assert_eq!(div_rem_euclid(-1, 2), (-1, 1));
        assert_eq!(div_rem_euclid(-1, 3), (-1, 2));

        assert_eq!(div_rem_euclid(0, 1), (0, 0));
        assert_eq!(div_rem_euclid(0, 2), (0, 0));
        assert_eq!(div_rem_euclid(0, 3), (0, 0));

        assert_eq!(div_rem_euclid(1, 1), (1, 0));
        assert_eq!(div_rem_euclid(1, 2), (0, 1));
        assert_eq!(div_rem_euclid(1, 3), (0, 1));

        assert_eq!(div_rem_euclid(2, 1), (2, 0));
        assert_eq!(div_rem_euclid(2, 2), (1, 0));
        assert_eq!(div_rem_euclid(2, 3), (0, 2));

        assert_eq!(div_rem_euclid(8, 1), (8, 0));
        assert_eq!(div_rem_euclid(8, 2), (4, 0));
        assert_eq!(div_rem_euclid(8, 3), (2, 2));

        assert_eq!(div_rem_euclid(9, 1), (9, 0));
        assert_eq!(div_rem_euclid(9, 2), (4, 1));
        assert_eq!(div_rem_euclid(9, 3), (3, 0));

        assert_eq!(div_rem_euclid(10, 1), (10, 0));
        assert_eq!(div_rem_euclid(10, 2), (5, 0));
        assert_eq!(div_rem_euclid(10, 3), (3, 1));

        assert_eq!(div_rem_euclid(i32::MAX, 1), (2147483647, 0));
        assert_eq!(div_rem_euclid(i32::MAX, 2), (1073741823, 1));
        assert_eq!(div_rem_euclid(i32::MAX, 3), (715827882, 1));
    }
}
pub mod indian {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::iso::Iso;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;

    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Indian;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct IndianDateInner(ArithmeticDate<Indian>);

    impl CalendarArithmetic for Indian {
        fn month_days(year: i32, month: u8) -> u8 {
            if month == 1 {
                if Self::is_leap_year(year) {
                    31
                } else {
                    30
                }
            } else if (2..=6).contains(&month) {
                31
            } else if (7..=12).contains(&month) {
                30
            } else {
                0
            }
        }

        fn months_for_every_year(_: i32) -> u8 {
            12
        }

        fn is_leap_year(year: i32) -> bool {
            Iso::is_leap_year(year + 78)
        }

        fn days_in_provided_year(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    const DAY_OFFSET: u32 = 80;
    const YEAR_OFFSET: i32 = 78;

    impl Calendar for Indian {
        type DateInner = IndianDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            if era.0 != tinystr!(16, "saka") {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            }

            ArithmeticDate::new_from_solar(self, year, month_code, day).map(IndianDateInner)
        }

        fn date_from_iso(&self, iso: Date<Iso>) -> IndianDateInner {
            let day_of_year_iso = Iso::day_of_year(*iso.inner());
            let mut year = iso.inner().0.year - YEAR_OFFSET;
            let day_of_year_indian = if day_of_year_iso <= DAY_OFFSET {
                year -= 1;
                let n_days = Self::days_in_provided_year(year);

                n_days + day_of_year_iso - DAY_OFFSET
            } else {
                day_of_year_iso - DAY_OFFSET
            };
            IndianDateInner(ArithmeticDate::date_from_year_day(year, day_of_year_indian))
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            let day_of_year_indian = date.0.day_of_year();
            let days_in_year = date.0.days_in_year();

            let mut year = date.0.year + YEAR_OFFSET;
            let day_of_year_iso = if day_of_year_indian + DAY_OFFSET >= days_in_year {
                year += 1;
                day_of_year_indian + DAY_OFFSET - days_in_year
            } else {
                day_of_year_indian + DAY_OFFSET
            };

            Iso::iso_from_year_day(year, day_of_year_iso)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            date.0.months_in_year()
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            date.0.days_in_year()
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            date.0.days_in_month()
        }

        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            Iso.day_of_week(Indian.date_to_iso(date).inner())
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            date.0.offset_date(offset);
        }

        #[allow(clippy::field_reassign_with_default)]
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            _largest_unit: DateDurationUnit,
            _smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            date1.0.until(date2.0, _largest_unit, _smallest_unit)
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "saka")),
                number: date.0.year,
                related_iso: None,
            }
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            date.0.day_of_month()
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = types::FormattableYear {
                era: types::Era(tinystr!(16, "saka")),
                number: date.0.year - 1,
                related_iso: None,
            };
            let next_year = types::FormattableYear {
                era: types::Era(tinystr!(16, "saka")),
                number: date.0.year + 1,
                related_iso: None,
            };
            types::DayOfYearInfo {
                day_of_year: date.0.day_of_year(),
                days_in_year: date.0.days_in_year(),
                prev_year,
                days_in_prev_year: Indian::days_in_year_direct(date.0.year - 1),
                next_year,
            }
        }

        fn debug_name(&self) -> &'static str {
            "Indian"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::Indian)
        }
    }

    impl Indian {
        pub fn new() -> Self {
            Self
        }

        fn days_in_year_direct(year: i32) -> u32 {
            if Indian::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Date<Indian> {
        pub fn try_new_indian_date(
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Indian>, CalendarError> {
            let inner = ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            };

            let bound = inner.days_in_month();
            if day > bound {
                return Err(CalendarError::OutOfRange);
            }

            Ok(Date::from_raw(IndianDateInner(inner), Indian))
        }
    }

    impl DateTime<Indian> {
        pub fn try_new_indian_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Indian>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_indian_date(year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn assert_roundtrip(y: i32, m: u8, d: u8, iso_y: i32, iso_m: u8, iso_d: u8) {
            let indian = Date::try_new_indian_date(y, m, d)
                .expect("Indian date should construct successfully");
            let iso = indian.to_iso();

            assert_eq!(
                iso.year().number,
                iso_y,
                "{y}-{m}-{d}: ISO year did not match"
            );
            assert_eq!(
                iso.month().ordinal as u8,
                iso_m,
                "{y}-{m}-{d}: ISO month did not match"
            );
            assert_eq!(
                iso.day_of_month().0 as u8,
                iso_d,
                "{y}-{m}-{d}: ISO day did not match"
            );

            let roundtrip = iso.to_calendar(Indian);

            assert_eq!(
                roundtrip.year().number,
                indian.year().number,
                "{y}-{m}-{d}: roundtrip year did not match"
            );
            assert_eq!(
                roundtrip.month().ordinal,
                indian.month().ordinal,
                "{y}-{m}-{d}: roundtrip month did not match"
            );
            assert_eq!(
                roundtrip.day_of_month(),
                indian.day_of_month(),
                "{y}-{m}-{d}: roundtrip day did not match"
            );
        }

        #[test]
        fn roundtrip_indian() {
            assert_roundtrip(1944, 6, 7, 2022, 8, 29);
            assert_roundtrip(1943, 6, 7, 2021, 8, 29);
            assert_roundtrip(1942, 6, 7, 2020, 8, 29);
            assert_roundtrip(1941, 6, 7, 2019, 8, 29);
            assert_roundtrip(1944, 11, 7, 2023, 1, 27);
            assert_roundtrip(1943, 11, 7, 2022, 1, 27);
            assert_roundtrip(1942, 11, 7, 2021, 1, 27);
            assert_roundtrip(1941, 11, 7, 2020, 1, 27);
        }
    }
}
pub mod iso {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::{div_rem_euclid, quotient};
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use tinystr::tinystr;

    const EPOCH: i32 = 1;


    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Iso;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct IsoDateInner(pub(crate) ArithmeticDate<Iso>);

    impl CalendarArithmetic for Iso {
        fn month_days(year: i32, month: u8) -> u8 {
            match month {
                4 | 6 | 9 | 11 => 30,
                2 if Self::is_leap_year(year) => 29,
                2 => 28,
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                _ => 0,
            }
        }

        fn months_for_every_year(_: i32) -> u8 {
            12
        }

        fn is_leap_year(year: i32) -> bool {
            year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)
        }

        fn days_in_provided_year(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Calendar for Iso {
        type DateInner = IsoDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            if era.0 != tinystr!(16, "default") {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            }

            ArithmeticDate::new_from_solar(self, year, month_code, day).map(IsoDateInner)
        }

        fn date_from_iso(&self, iso: Date<Iso>) -> IsoDateInner {
            *iso.inner()
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            Date::from_raw(*date, Iso)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            date.0.months_in_year()
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            date.0.days_in_year()
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            date.0.days_in_month()
        }

        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {

            let years_since_400 = date.0.year % 400;
            let leap_years_since_400 = years_since_400 / 4 - years_since_400 / 100;
            let days_to_current_year = 365 * years_since_400 + leap_years_since_400;
            let year_offset = days_to_current_year % 7;

            let month_offset = if Self::is_leap_year(date.0.year) {
                match date.0.month {
                    10 => 0,
                    5 => 1,
                    2 | 8 => 2,
                    3 | 11 => 3,
                    6 => 4,
                    9 | 12 => 5,
                    1 | 4 | 7 => 6,
                    _ => unreachable!(),
                }
            } else {
                match date.0.month {
                    1 | 10 => 0,
                    5 => 1,
                    8 => 2,
                    2 | 3 | 11 => 3,
                    6 => 4,
                    9 | 12 => 5,
                    4 | 7 => 6,
                    _ => unreachable!(),
                }
            };
            let january_1_2000 = 5; // Saturday
            let day_offset = (january_1_2000 + year_offset + month_offset + date.0.day as i32) % 7;

            types::IsoWeekday::from((day_offset + 1) as usize)
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            date.0.offset_date(offset);
        }

        #[allow(clippy::field_reassign_with_default)]
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            _largest_unit: DateDurationUnit,
            _smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            date1.0.until(date2.0, _largest_unit, _smallest_unit)
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            Self::year_as_iso(date.0.year)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            date.0.day_of_month()
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = date.0.year - 1;
            let next_year = date.0.year + 1;
            types::DayOfYearInfo {
                day_of_year: date.0.day_of_year(),
                days_in_year: date.0.days_in_year(),
                prev_year: Self::year_as_iso(prev_year),
                days_in_prev_year: Iso::days_in_year_direct(prev_year),
                next_year: Self::year_as_iso(next_year),
            }
        }

        fn debug_name(&self) -> &'static str {
            "ISO"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::Iso)
        }
    }

    impl Date<Iso> {
        pub fn try_new_iso_date(year: i32, month: u8, day: u8) -> Result<Date<Iso>, CalendarError> {
            if !(1..=12).contains(&month) {
                return Err(CalendarError::OutOfRange);
            }
            if day == 0 || day > Iso::days_in_month(year, month) {
                return Err(CalendarError::OutOfRange);
            }
            Ok(Date::from_raw(
                IsoDateInner(ArithmeticDate::new(year, month, day)),
                Iso,
            ))
        }
    }

    impl DateTime<Iso> {
        pub fn try_new_iso_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Iso>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_iso_date(year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }

        pub fn minutes_since_local_unix_epoch(&self) -> i32 {
            let minutes_a_hour = 60;
            let hours_a_day = 24;
            let minutes_a_day = minutes_a_hour * hours_a_day;
            if let Ok(unix_epoch) = DateTime::try_new_iso_datetime(1970, 1, 1, 0, 0, 0) {
                (Iso::fixed_from_iso(*self.date.inner())
                    - Iso::fixed_from_iso(*unix_epoch.date.inner()))
                    * minutes_a_day
                    + i32::from(self.time.hour.number()) * minutes_a_hour
                    + i32::from(self.time.minute.number())
            } else {
                unreachable!("DateTime should be created successfully")
            }
        }

        pub fn from_minutes_since_local_unix_epoch(minute: i32) -> DateTime<Iso> {
            let (time, extra_days) = types::Time::from_minute_with_remainder_days(minute);
            #[allow(clippy::unwrap_used)] // constant date
            let unix_epoch = DateTime::try_new_iso_datetime(1970, 1, 1, 0, 0, 0).unwrap();
            let unix_epoch_days = Iso::fixed_from_iso(*unix_epoch.date.inner());
            let date = Iso::iso_from_fixed(unix_epoch_days + extra_days);
            DateTime { date, time }
        }
    }

    impl Iso {
        pub fn new() -> Self {
            Self
        }

        fn days_in_month(year: i32, month: u8) -> u8 {
            match month {
                4 | 6 | 9 | 11 => 30,
                2 if Self::is_leap_year(year) => 29,
                2 => 28,
                _ => 31,
            }
        }

        pub(crate) fn days_in_year_direct(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }

        pub(crate) fn fixed_from_iso(date: IsoDateInner) -> i32 {
            let mut fixed: i32 = EPOCH - 1 + 365 * (date.0.year - 1);
            fixed += quotient(date.0.year - 1, 4) - quotient(date.0.year - 1, 100)
                + quotient(date.0.year - 1, 400);
            fixed += quotient(367 * (date.0.month as i32) - 362, 12);
            fixed += if date.0.month <= 2 {
                0
            } else if Self::is_leap_year(date.0.year) {
                -1
            } else {
                -2
            };
            fixed + (date.0.day as i32)
        }

        fn fixed_from_iso_integers(year: i32, month: u8, day: u8) -> Option<i32> {
            Date::try_new_iso_date(year, month, day)
                .ok()
                .map(|d| *d.inner())
                .map(Self::fixed_from_iso)
        }

        pub(crate) fn iso_from_year_day(year: i32, year_day: u32) -> Date<Iso> {
            let mut month = 1;
            let mut day = year_day as i32;
            while month <= 12 {
                let month_days = Self::days_in_month(year, month) as i32;
                if day <= month_days {
                    break;
                } else {
                    debug_assert!(month < 12); // don't try going to month 13
                    day -= month_days;
                    month += 1;
                }
            }
            let day = day as u8; // day <= month_days < u8::MAX

            #[allow(clippy::unwrap_used)] // month in 1..=12, day <= month_days
            Date::try_new_iso_date(year, month, day).unwrap()
        }

        fn iso_year_from_fixed(date: i32) -> i32 {
            let date = date - EPOCH;
            let (n_400, date) = div_rem_euclid(date, 146097);

            let (n_100, date) = div_rem_euclid(date, 36524);

            let (n_4, date) = div_rem_euclid(date, 1461);

            let n_1 = quotient(date, 365);

            let year = 400 * n_400 + 100 * n_100 + 4 * n_4 + n_1;

            if n_100 == 4 || n_1 == 4 {
                year
            } else {
                year + 1
            }
        }

        fn iso_new_year(year: i32) -> i32 {
            #[allow(clippy::unwrap_used)] // valid day and month
            Self::fixed_from_iso_integers(year, 1, 1).unwrap()
        }

        pub(crate) fn iso_from_fixed(date: i32) -> Date<Iso> {
            let year = Self::iso_year_from_fixed(date);
            let prior_days = date - Self::iso_new_year(year);
            #[allow(clippy::unwrap_used)] // valid day and month
            let correction = if date < Self::fixed_from_iso_integers(year, 3, 1).unwrap() {
                0
            } else if Self::is_leap_year(year) {
                1
            } else {
                2
            };
            let month = quotient(12 * (prior_days + correction) + 373, 367) as u8; // in 1..12 < u8::MAX
            #[allow(clippy::unwrap_used)] // valid day and month
            let day = (date - Self::fixed_from_iso_integers(year, month, 1).unwrap() + 1) as u8; // <= days_in_month < u8::MAX
            #[allow(clippy::unwrap_used)] // valid day and month
            Date::try_new_iso_date(year, month, day).unwrap()
        }

        pub(crate) fn day_of_year(date: IsoDateInner) -> u32 {
            let month_offset = [0, 1, -1, 0, 0, 1, 1, 2, 3, 3, 4, 4];
            #[allow(clippy::indexing_slicing)] // date.0.month in 1..=12
            let mut offset = month_offset[date.0.month as usize - 1];
            if Self::is_leap_year(date.0.year) && date.0.month > 2 {
                offset += 1;
            }
            let prev_month_days = (30 * (date.0.month as i32 - 1) + offset) as u32;

            prev_month_days + date.0.day as u32
        }

        fn year_as_iso(year: i32) -> types::FormattableYear {
            types::FormattableYear {
                era: types::Era(tinystr!(16, "default")),
                number: year,
                related_iso: None,
            }
        }
    }

    impl IsoDateInner {
        pub(crate) fn jan_1(year: i32) -> Self {
            Self(ArithmeticDate::new(year, 1, 1))
        }
        pub(crate) fn dec_31(year: i32) -> Self {
            Self(ArithmeticDate::new(year, 12, 1))
        }
    }

    impl From<&'_ IsoDateInner> for crate::provider::EraStartDate {
        fn from(other: &'_ IsoDateInner) -> Self {
            Self {
                year: other.0.year,
                month: other.0.month,
                day: other.0.day,
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::types::IsoWeekday;

        #[test]
        fn test_day_of_week() {
            assert_eq!(
                Date::try_new_iso_date(2021, 6, 23).unwrap().day_of_week(),
                IsoWeekday::Wednesday,
            );
            assert_eq!(
                Date::try_new_iso_date(1983, 2, 2).unwrap().day_of_week(),
                IsoWeekday::Wednesday,
            );
            assert_eq!(
                Date::try_new_iso_date(2020, 1, 21).unwrap().day_of_week(),
                IsoWeekday::Tuesday,
            );
        }

        #[test]
        fn test_day_of_year() {
            assert_eq!(
                Date::try_new_iso_date(2021, 6, 23)
                    .unwrap()
                    .day_of_year_info()
                    .day_of_year,
                174,
            );
            assert_eq!(
                Date::try_new_iso_date(2020, 6, 23)
                    .unwrap()
                    .day_of_year_info()
                    .day_of_year,
                175,
            );
            assert_eq!(
                Date::try_new_iso_date(1983, 2, 2)
                    .unwrap()
                    .day_of_year_info()
                    .day_of_year,
                33,
            );
        }

        fn simple_subtract(a: &Date<Iso>, b: &Date<Iso>) -> DateDuration<Iso> {
            let a = a.inner();
            let b = b.inner();
            DateDuration::new(
                a.0.year - b.0.year,
                a.0.month as i32 - b.0.month as i32,
                0,
                a.0.day as i32 - b.0.day as i32,
            )
        }

        #[test]
        fn test_offset() {
            let today = Date::try_new_iso_date(2021, 6, 23).unwrap();
            let today_plus_5000 = Date::try_new_iso_date(2035, 3, 2).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 5000));
            assert_eq!(offset, today_plus_5000);
            let offset = today.added(simple_subtract(&today_plus_5000, &today));
            assert_eq!(offset, today_plus_5000);

            let today = Date::try_new_iso_date(2021, 6, 23).unwrap();
            let today_minus_5000 = Date::try_new_iso_date(2007, 10, 15).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, -5000));
            assert_eq!(offset, today_minus_5000);
            let offset = today.added(simple_subtract(&today_minus_5000, &today));
            assert_eq!(offset, today_minus_5000);
        }

        #[test]
        fn test_offset_at_month_boundary() {
            let today = Date::try_new_iso_date(2020, 2, 28).unwrap();
            let today_plus_2 = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 2));
            assert_eq!(offset, today_plus_2);

            let today = Date::try_new_iso_date(2020, 2, 28).unwrap();
            let today_plus_3 = Date::try_new_iso_date(2020, 3, 2).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 3));
            assert_eq!(offset, today_plus_3);

            let today = Date::try_new_iso_date(2020, 2, 28).unwrap();
            let today_plus_1 = Date::try_new_iso_date(2020, 2, 29).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 1));
            assert_eq!(offset, today_plus_1);

            let today = Date::try_new_iso_date(2019, 2, 28).unwrap();
            let today_plus_2 = Date::try_new_iso_date(2019, 3, 2).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 2));
            assert_eq!(offset, today_plus_2);

            let today = Date::try_new_iso_date(2019, 2, 28).unwrap();
            let today_plus_1 = Date::try_new_iso_date(2019, 3, 1).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, 1));
            assert_eq!(offset, today_plus_1);

            let today = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let today_minus_1 = Date::try_new_iso_date(2020, 2, 29).unwrap();
            let offset = today.added(DateDuration::new(0, 0, 0, -1));
            assert_eq!(offset, today_minus_1);
        }

        #[test]
        fn test_offset_handles_negative_month_offset() {
            let today = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let today_minus_2_months = Date::try_new_iso_date(2020, 1, 1).unwrap();
            let offset = today.added(DateDuration::new(0, -2, 0, 0));
            assert_eq!(offset, today_minus_2_months);

            let today = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let today_minus_4_months = Date::try_new_iso_date(2019, 11, 1).unwrap();
            let offset = today.added(DateDuration::new(0, -4, 0, 0));
            assert_eq!(offset, today_minus_4_months);

            let today = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let today_minus_24_months = Date::try_new_iso_date(2018, 3, 1).unwrap();
            let offset = today.added(DateDuration::new(0, -24, 0, 0));
            assert_eq!(offset, today_minus_24_months);

            let today = Date::try_new_iso_date(2020, 3, 1).unwrap();
            let today_minus_27_months = Date::try_new_iso_date(2017, 12, 1).unwrap();
            let offset = today.added(DateDuration::new(0, -27, 0, 0));
            assert_eq!(offset, today_minus_27_months);
        }

        #[test]
        fn test_offset_handles_out_of_bound_month_offset() {
            let today = Date::try_new_iso_date(2021, 1, 31).unwrap();
            let today_plus_1_month = Date::try_new_iso_date(2021, 3, 3).unwrap();
            let offset = today.added(DateDuration::new(0, 1, 0, 0));
            assert_eq!(offset, today_plus_1_month);

            let today = Date::try_new_iso_date(2021, 1, 31).unwrap();
            let today_plus_1_month_1_day = Date::try_new_iso_date(2021, 3, 4).unwrap();
            let offset = today.added(DateDuration::new(0, 1, 0, 1));
            assert_eq!(offset, today_plus_1_month_1_day);
        }

        #[test]
        fn test_iso_to_from_fixed() {
            fn check(fixed: i32, year: i32, month: u8, day: u8) {
                assert_eq!(Iso::iso_year_from_fixed(fixed), year, "fixed: {fixed}");
                assert_eq!(
                    Iso::iso_from_fixed(fixed),
                    Date::try_new_iso_date(year, month, day).unwrap(),
                    "fixed: {fixed}"
                );
                assert_eq!(
                    Iso::fixed_from_iso_integers(year, month, day),
                    Some(fixed),
                    "fixed: {fixed}"
                );
            }
            check(-1828, -5, 12, 30);
            check(-1827, -5, 12, 31); // leap year
            check(-1826, -4, 1, 1);
            check(-1462, -4, 12, 30);
            check(-1461, -4, 12, 31);
            check(-1460, -3, 1, 1);
            check(-1459, -3, 1, 2);
            check(-732, -2, 12, 30);
            check(-731, -2, 12, 31);
            check(-730, -1, 1, 1);
            check(-367, -1, 12, 30);
            check(-366, -1, 12, 31);
            check(-365, 0, 1, 1); // leap year
            check(-364, 0, 1, 2);
            check(-1, 0, 12, 30);
            check(0, 0, 12, 31);
            check(1, 1, 1, 1);
            check(2, 1, 1, 2);
            check(364, 1, 12, 30);
            check(365, 1, 12, 31);
            check(366, 2, 1, 1);
            check(1459, 4, 12, 29);
            check(1460, 4, 12, 30);
            check(1461, 4, 12, 31); // leap year
            check(1462, 5, 1, 1);
        }

        #[test]
        fn test_from_minutes_since_local_unix_epoch() {
            fn check(minutes: i32, year: i32, month: u8, day: u8, hour: u8, minute: u8) {
                let today =
                    DateTime::try_new_iso_datetime(year, month, day, hour, minute, 0).unwrap();
                assert_eq!(today.minutes_since_local_unix_epoch(), minutes);
                assert_eq!(
                    DateTime::from_minutes_since_local_unix_epoch(minutes),
                    today
                );
            }

            check(-1441, 1969, 12, 30, 23, 59);
            check(-1440, 1969, 12, 31, 0, 0);
            check(-1439, 1969, 12, 31, 0, 1);
            check(-2879, 1969, 12, 30, 0, 1);
        }
    }
}
pub mod japanese {


    use crate::any_calendar::AnyCalendarKind;
    use crate::iso::{Iso, IsoDateInner};
    use crate::provider::{EraStartDate, JapaneseErasV1Marker, JapaneseExtendedErasV1Marker};
    use crate::{
        types, AsCalendar, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime,
        Ref,
    };
    use icu_provider::prelude::*;
    use tinystr::{tinystr, TinyStr16};

    #[derive(Clone, Debug, Default)]
    pub struct Japanese {
        eras: DataPayload<JapaneseErasV1Marker>,
    }

    #[derive(Clone, Debug, Default)]
    pub struct JapaneseExtended(Japanese);

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct JapaneseDateInner {
        inner: IsoDateInner,
        adjusted_year: i32,
        era: TinyStr16,
    }

    impl Japanese {
        pub fn try_new_unstable<D: DataProvider<JapaneseErasV1Marker> + ?Sized>(
            data_provider: &D,
        ) -> Result<Self, CalendarError> {
            let eras = data_provider
                .load(DataRequest {
                    locale: Default::default(),
                    metadata: Default::default(),
                })?
                .take_payload()?;
            Ok(Self { eras })
        }

        icu_provider::gen_any_buffer_constructors!(
            locale: skip,
            options: skip,
            error: CalendarError
        );

        fn japanese_date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
            debug_name: &'static str,
        ) -> Result<JapaneseDateInner, CalendarError> {
            let month = crate::calendar_arithmetic::ordinal_solar_month_from_code(month_code);
            let month = if let Some(month) = month {
                month
            } else {
                return Err(CalendarError::UnknownMonthCode(month_code.0, debug_name));
            };

            if month > 12 {
                return Err(CalendarError::UnknownMonthCode(month_code.0, debug_name));
            }

            self.new_japanese_date_inner(era, year, month, day)
        }
    }

    impl JapaneseExtended {
        pub fn try_new_unstable<D: DataProvider<JapaneseExtendedErasV1Marker> + ?Sized>(
            data_provider: &D,
        ) -> Result<Self, CalendarError> {
            let eras = data_provider
                .load(DataRequest {
                    locale: Default::default(),
                    metadata: Default::default(),
                })?
                .take_payload()?;
            Ok(Self(Japanese { eras: eras.cast() }))
        }

        icu_provider::gen_any_buffer_constructors!(
            locale: skip,
            options: skip,
            error: CalendarError
        );
    }

    impl Calendar for Japanese {
        type DateInner = JapaneseDateInner;

        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            self.japanese_date_from_codes(era, year, month_code, day, self.debug_name())
        }

        fn date_from_iso(&self, iso: Date<Iso>) -> JapaneseDateInner {
            let (adjusted_year, era) = self.adjusted_year_for(iso.inner());
            JapaneseDateInner {
                inner: *iso.inner(),
                adjusted_year,
                era,
            }
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            Date::from_raw(date.inner, Iso)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            Iso.months_in_year(&date.inner)
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            Iso.days_in_year(&date.inner)
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            Iso.days_in_month(&date.inner)
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            Iso.offset_date(&mut date.inner, offset.cast_unit());
            let (adjusted_year, era) = self.adjusted_year_for(&date.inner);
            date.adjusted_year = adjusted_year;
            date.era = era
        }

        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            Iso.until(
                &date1.inner,
                &date2.inner,
                &Iso,
                largest_unit,
                smallest_unit,
            )
            .cast_unit()
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            types::FormattableYear {
                era: types::Era(date.era),
                number: date.adjusted_year,
                related_iso: None,
            }
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            Iso.month(&date.inner)
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            Iso.day_of_month(&date.inner)
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_dec_31 = IsoDateInner::dec_31(date.inner.0.year - 1);
            let next_jan_1 = IsoDateInner::jan_1(date.inner.0.year + 1);

            let prev_dec_31 = self.date_from_iso(Date::from_raw(prev_dec_31, Iso));
            let next_jan_1 = self.date_from_iso(Date::from_raw(next_jan_1, Iso));
            types::DayOfYearInfo {
                day_of_year: Iso::days_in_year_direct(date.inner.0.year),
                days_in_year: Iso::days_in_year_direct(date.inner.0.year),
                prev_year: self.year(&prev_dec_31),
                days_in_prev_year: Iso::days_in_year_direct(prev_dec_31.inner.0.year),
                next_year: self.year(&next_jan_1),
            }
        }

        fn debug_name(&self) -> &'static str {
            "Japanese (Modern eras only)"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::Japanese)
        }
    }

    impl Calendar for JapaneseExtended {
        type DateInner = JapaneseDateInner;

        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            self.0
                .japanese_date_from_codes(era, year, month_code, day, self.debug_name())
        }

        fn date_from_iso(&self, iso: Date<Iso>) -> JapaneseDateInner {
            Japanese::date_from_iso(&self.0, iso)
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            Japanese::date_to_iso(&self.0, date)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            Japanese::months_in_year(&self.0, date)
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            Japanese::days_in_year(&self.0, date)
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            Japanese::days_in_month(&self.0, date)
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            Japanese::offset_date(&self.0, date, offset.cast_unit())
        }

        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            Japanese::until(
                &self.0,
                date1,
                date2,
                &calendar2.0,
                largest_unit,
                smallest_unit,
            )
            .cast_unit()
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            Japanese::year(&self.0, date)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            Japanese::month(&self.0, date)
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            Japanese::day_of_month(&self.0, date)
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            Japanese::day_of_year_info(&self.0, date)
        }

        fn debug_name(&self) -> &'static str {
            "Japanese (With historical eras)"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            Some(AnyCalendarKind::JapaneseExtended)
        }
    }

    impl Date<Japanese> {
        pub fn try_new_japanese_date<A: AsCalendar<Calendar = Japanese>>(
            era: types::Era,
            year: i32,
            month: u8,
            day: u8,
            japanese_calendar: A,
        ) -> Result<Date<A>, CalendarError> {
            let inner = japanese_calendar
                .as_calendar()
                .new_japanese_date_inner(era, year, month, day)?;
            Ok(Date::from_raw(inner, japanese_calendar))
        }
    }

    impl Date<JapaneseExtended> {
        pub fn try_new_japanese_extended_date<A: AsCalendar<Calendar = JapaneseExtended>>(
            era: types::Era,
            year: i32,
            month: u8,
            day: u8,
            japanext_calendar: A,
        ) -> Result<Date<A>, CalendarError> {
            let inner = japanext_calendar
                .as_calendar()
                .0
                .new_japanese_date_inner(era, year, month, day)?;
            Ok(Date::from_raw(inner, japanext_calendar))
        }

        #[doc(hidden)]
        pub fn into_japanese_date(self) -> Date<Japanese> {
            Date::from_raw(self.inner, self.calendar.0)
        }
    }

    impl DateTime<Japanese> {
        #[allow(clippy::too_many_arguments)] // it's more convenient to have this many arguments
        pub fn try_new_japanese_datetime<A: AsCalendar<Calendar = Japanese>>(
            era: types::Era,
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
            japanese_calendar: A,
        ) -> Result<DateTime<A>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_japanese_date(era, year, month, day, japanese_calendar)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    impl DateTime<JapaneseExtended> {
        #[allow(clippy::too_many_arguments)] // it's more convenient to have this many arguments
        pub fn try_new_japanese_extended_datetime<A: AsCalendar<Calendar = JapaneseExtended>>(
            era: types::Era,
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
            japanext_calendar: A,
        ) -> Result<DateTime<A>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_japanese_extended_date(
                    era,
                    year,
                    month,
                    day,
                    japanext_calendar,
                )?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    const MEIJI_START: EraStartDate = EraStartDate {
        year: 1868,
        month: 9,
        day: 8,
    };
    const TAISHO_START: EraStartDate = EraStartDate {
        year: 1912,
        month: 7,
        day: 30,
    };
    const SHOWA_START: EraStartDate = EraStartDate {
        year: 1926,
        month: 12,
        day: 25,
    };
    const HEISEI_START: EraStartDate = EraStartDate {
        year: 1989,
        month: 1,
        day: 8,
    };
    const REIWA_START: EraStartDate = EraStartDate {
        year: 2019,
        month: 5,
        day: 1,
    };

    const FALLBACK_ERA: (EraStartDate, TinyStr16) = (REIWA_START, tinystr!(16, "reiwa"));

    impl Japanese {
        fn adjusted_year_for(&self, date: &IsoDateInner) -> (i32, TinyStr16) {
            let date: EraStartDate = date.into();
            let (start, era) = self.japanese_era_for(date);
            if date < start {
                if date.year < 0 {
                    (1 - date.year, tinystr!(16, "bce"))
                } else {
                    (date.year, tinystr!(16, "ce"))
                }
            } else {
                (date.year - start.year + 1, era)
            }
        }

        fn japanese_era_for(&self, date: EraStartDate) -> (EraStartDate, TinyStr16) {
            let era_data = self.eras.get();
            if date >= MEIJI_START
                && era_data.dates_to_eras.last().map(|x| x.1) == Some(tinystr!(16, "reiwa"))
            {
                return if date >= REIWA_START {
                    (REIWA_START, tinystr!(16, "reiwa"))
                } else if date >= HEISEI_START {
                    (HEISEI_START, tinystr!(16, "heisei"))
                } else if date >= SHOWA_START {
                    (SHOWA_START, tinystr!(16, "showa"))
                } else if date >= TAISHO_START {
                    (TAISHO_START, tinystr!(16, "taisho"))
                } else {
                    (MEIJI_START, tinystr!(16, "meiji"))
                };
            }
            let data = &era_data.dates_to_eras;
            match data.binary_search_by(|(d, _)| d.cmp(&date)) {
                Ok(index) => data.get(index),
                Err(index) if index == 0 => data.get(index),
                Err(index) => data.get(index - 1).or_else(|| data.iter().next_back()),
            }
            .unwrap_or(FALLBACK_ERA)
        }

        fn japanese_era_range_for(
            &self,
            era: TinyStr16,
        ) -> Result<(EraStartDate, Option<EraStartDate>), CalendarError> {
            if era == tinystr!(16, "reiwa") {
                if let Some(last) = self.eras.get().dates_to_eras.last() {
                    if last.1 == era {
                        return Ok((REIWA_START, None));
                    }
                }
            } else if era == tinystr!(16, "heisei") {
                return Ok((HEISEI_START, Some(REIWA_START)));
            } else if era == tinystr!(16, "showa") {
                return Ok((SHOWA_START, Some(HEISEI_START)));
            } else if era == tinystr!(16, "taisho") {
                return Ok((TAISHO_START, Some(SHOWA_START)));
            } else if era == tinystr!(16, "meiji") {
                return Ok((MEIJI_START, Some(TAISHO_START)));
            }

            let era_data = self.eras.get();
            let data = &era_data.dates_to_eras;
            if let Some(year) = era.split('-').nth(1) {
                if let Ok(ref int) = year.parse::<i32>() {
                    if let Ok(index) = data.binary_search_by(|(d, _)| d.year.cmp(int)) {
                        #[allow(clippy::expect_used)] // see expect message
                        let (era_start, code) = data
                            .get(index)
                            .expect("Indexing from successful binary search must succeed");
                        if code == era {
                            return Ok((era_start, data.get(index + 1).map(|e| e.0)));
                        }
                    }
                }
            }

            if let Some((index, (start, _))) = data.iter().enumerate().rev().find(|d| d.1 .1 == era)
            {
                return Ok((start, data.get(index + 1).map(|e| e.0)));
            }

            Err(CalendarError::UnknownEra(era, self.debug_name()))
        }

        fn new_japanese_date_inner(
            &self,
            era: types::Era,
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<JapaneseDateInner, CalendarError> {
            let cal = Ref(self);
            if era.0 == tinystr!(16, "bce") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                return Ok(Date::try_new_iso_date(1 - year, month, day)?
                    .to_calendar(cal)
                    .inner);
            } else if era.0 == tinystr!(16, "ce") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                return Ok(Date::try_new_iso_date(year, month, day)?
                    .to_calendar(cal)
                    .inner);
            }

            let (era_start, next_era_start) = self.japanese_era_range_for(era.0)?;

            let date_in_iso = EraStartDate {
                year: era_start.year + year - 1,
                month,
                day,
            };

            if date_in_iso < era_start {
                return Err(CalendarError::OutOfRange);
            } else if let Some(next_era_start) = next_era_start {
                if date_in_iso >= next_era_start {
                    return Err(CalendarError::OutOfRange);
                }
            }

            let iso = Date::try_new_iso_date(date_in_iso.year, date_in_iso.month, date_in_iso.day)?;
            Ok(JapaneseDateInner {
                inner: iso.inner,
                adjusted_year: year,
                era: era.0,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::Ref;

        fn single_test_roundtrip(
            calendar: Ref<Japanese>,
            era: &str,
            year: i32,
            month: u8,
            day: u8,
        ) {
            let era = types::Era(era.parse().expect("era must parse"));

            let date =
                Date::try_new_japanese_date(era, year, month, day, calendar).unwrap_or_else(|e| {
                    panic!("Failed to construct date with {era:?}, {year}, {month}, {day}: {e}")
                });
            let iso = date.to_iso();
            let reconstructed = Date::new_from_iso(iso, calendar);
            assert_eq!(
                date, reconstructed,
                "Failed to roundtrip with {era:?}, {year}, {month}, {day}"
            )
        }

        fn single_test_roundtrip_ext(
            calendar: Ref<JapaneseExtended>,
            era: &str,
            year: i32,
            month: u8,
            day: u8,
        ) {
            let era = types::Era(era.parse().expect("era must parse"));

            let date = Date::try_new_japanese_extended_date(era, year, month, day, calendar)
                .unwrap_or_else(|e| {
                    panic!("Failed to construct date with {era:?}, {year}, {month}, {day}: {e}")
                });
            let iso = date.to_iso();
            let reconstructed = Date::new_from_iso(iso, calendar);
            assert_eq!(
                date, reconstructed,
                "Failed to roundtrip with {era:?}, {year}, {month}, {day}"
            )
        }

        fn single_test_gregorian_roundtrip_ext(
            calendar: Ref<JapaneseExtended>,
            era: &str,
            year: i32,
            month: u8,
            day: u8,
            era2: &str,
            year2: i32,
        ) {
            let era = types::Era(era.parse().expect("era must parse"));
            let era2 = types::Era(era2.parse().expect("era must parse"));

            let expected = Date::try_new_japanese_extended_date(era2, year2, month, day, calendar)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to construct expectation date with {era2:?}, {year2}, {month}, {day}: {e}"
                )
            });

            let date = Date::try_new_japanese_extended_date(era, year, month, day, calendar)
                .unwrap_or_else(|e| {
                    panic!("Failed to construct date with {era:?}, {year}, {month}, {day}: {e}")
                });
            let iso = date.to_iso();
            let reconstructed = Date::new_from_iso(iso, calendar);
            assert_eq!(
                expected, reconstructed,
                "Failed to roundtrip with {era:?}, {year}, {month}, {day} == {era2:?}, {year}"
            )
        }

        fn single_test_error(
            calendar: Ref<Japanese>,
            era: &str,
            year: i32,
            month: u8,
            day: u8,
            error: CalendarError,
        ) {
            let era = types::Era(era.parse().expect("era must parse"));

            let date = Date::try_new_japanese_date(era, year, month, day, calendar);
            assert_eq!(
                date,
                Err(error),
                "Construction with {era:?}, {year}, {month}, {day} did not return {error:?}"
            )
        }

        fn single_test_error_ext(
            calendar: Ref<JapaneseExtended>,
            era: &str,
            year: i32,
            month: u8,
            day: u8,
            error: CalendarError,
        ) {
            let era = types::Era(era.parse().expect("era must parse"));

            let date = Date::try_new_japanese_extended_date(era, year, month, day, calendar);
            assert_eq!(
                date,
                Err(error),
                "Construction with {era:?}, {year}, {month}, {day} did not return {error:?}"
            )
        }

        #[test]
        #[cfg(feature = "serde")]
        fn test_japanese() {
            let calendar = Japanese::try_new_unstable(&icu_testdata::buffer().as_deserializing())
                .expect("Cannot load japanese data");
            let calendar_ext =
                JapaneseExtended::try_new_unstable(&icu_testdata::buffer().as_deserializing())
                    .expect("Cannot load japanese data");
            let calendar = Ref(&calendar);
            let calendar_ext = Ref(&calendar_ext);

            single_test_roundtrip(calendar, "heisei", 12, 3, 1);
            single_test_roundtrip(calendar, "taisho", 3, 3, 1);
            single_test_error(calendar, "heisei", 1, 1, 1, CalendarError::OutOfRange);

            single_test_roundtrip_ext(calendar_ext, "heisei", 12, 3, 1);
            single_test_roundtrip_ext(calendar_ext, "taisho", 3, 3, 1);
            single_test_error_ext(calendar_ext, "heisei", 1, 1, 1, CalendarError::OutOfRange);

            single_test_roundtrip_ext(calendar_ext, "hakuho-672", 4, 3, 1);
            single_test_error(
                calendar,
                "hakuho-672",
                4,
                3,
                1,
                CalendarError::UnknownEra(
                    "hakuho-672".parse().unwrap(),
                    "Japanese (Modern eras only)",
                ),
            );

            single_test_roundtrip(calendar, "bce", 100, 3, 1);
            single_test_roundtrip(calendar, "bce", 1, 3, 1);
            single_test_roundtrip(calendar, "ce", 1, 3, 1);
            single_test_roundtrip(calendar, "ce", 100, 3, 1);
            single_test_roundtrip_ext(calendar_ext, "ce", 100, 3, 1);
            single_test_roundtrip(calendar, "ce", 1000, 3, 1);
            single_test_error(calendar, "ce", 0, 3, 1, CalendarError::OutOfRange);
            single_test_error(calendar, "bce", -1, 3, 1, CalendarError::OutOfRange);

            single_test_gregorian_roundtrip_ext(calendar_ext, "ce", 1000, 3, 1, "choho-999", 2);
            single_test_gregorian_roundtrip_ext(
                calendar_ext,
                "ce",
                749,
                5,
                10,
                "tenpyokampo-749",
                1,
            );
            single_test_gregorian_roundtrip_ext(calendar_ext, "bce", 10, 3, 1, "bce", 10);

            single_test_roundtrip_ext(calendar_ext, "tenpyokampo-749", 1, 4, 20);
            single_test_roundtrip_ext(calendar_ext, "tenpyokampo-749", 1, 4, 14);
            single_test_roundtrip_ext(calendar_ext, "tenpyokampo-749", 1, 7, 1);
            single_test_error_ext(
                calendar_ext,
                "tenpyokampo-749",
                1,
                7,
                5,
                CalendarError::OutOfRange,
            );
            single_test_error_ext(
                calendar_ext,
                "tenpyokampo-749",
                1,
                4,
                13,
                CalendarError::OutOfRange,
            );
        }
    }
}
pub mod julian {


    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::quotient;
    use crate::iso::Iso;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;

    const JULIAN_EPOCH: i32 = -1;

    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Julian;

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub struct JulianDateInner(pub(crate) ArithmeticDate<Julian>);

    impl CalendarArithmetic for Julian {
        fn month_days(year: i32, month: u8) -> u8 {
            match month {
                4 | 6 | 9 | 11 => 30,
                2 if Self::is_leap_year(year) => 29,
                2 => 28,
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                _ => 0,
            }
        }

        fn months_for_every_year(_: i32) -> u8 {
            12
        }

        fn is_leap_year(year: i32) -> bool {
            Self::is_leap_year_const(year)
        }

        fn days_in_provided_year(year: i32) -> u32 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    }

    impl Calendar for Julian {
        type DateInner = JulianDateInner;
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError> {
            let year = if era.0 == tinystr!(16, "ad") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                year
            } else if era.0 == tinystr!(16, "bc") {
                if year <= 0 {
                    return Err(CalendarError::OutOfRange);
                }
                1 - year
            } else {
                return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
            };

            ArithmeticDate::new_from_solar(self, year, month_code, day).map(JulianDateInner)
        }
        fn date_from_iso(&self, iso: Date<Iso>) -> JulianDateInner {
            let fixed_iso = Iso::fixed_from_iso(*iso.inner());
            Self::julian_from_fixed(fixed_iso)
        }

        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
            let fixed_julian = Julian::fixed_from_julian(date.0);
            Iso::iso_from_fixed(fixed_julian)
        }

        fn months_in_year(&self, date: &Self::DateInner) -> u8 {
            date.0.months_in_year()
        }

        fn days_in_year(&self, date: &Self::DateInner) -> u32 {
            date.0.days_in_year()
        }

        fn days_in_month(&self, date: &Self::DateInner) -> u8 {
            date.0.days_in_month()
        }

        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            Iso.day_of_week(Julian.date_to_iso(date).inner())
        }

        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
            date.0.offset_date(offset);
        }

        #[allow(clippy::field_reassign_with_default)]
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            _calendar2: &Self,
            _largest_unit: DateDurationUnit,
            _smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self> {
            date1.0.until(date2.0, _largest_unit, _smallest_unit)
        }

        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            crate::gregorian::year_as_gregorian(date.0.year)
        }

        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }

        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
            date.0.day_of_month()
        }

        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
            let prev_year = date.0.year - 1;
            let next_year = date.0.year + 1;
            types::DayOfYearInfo {
                day_of_year: date.0.day_of_year(),
                days_in_year: date.0.days_in_year(),
                prev_year: crate::gregorian::year_as_gregorian(prev_year),
                days_in_prev_year: Julian::days_in_year_direct(prev_year),
                next_year: crate::gregorian::year_as_gregorian(next_year),
            }
        }

        fn debug_name(&self) -> &'static str {
            "Julian"
        }

        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            None
        }
    }

    impl Julian {
        pub fn new() -> Self {
            Self
        }

        #[inline(always)]
        const fn is_leap_year_const(year: i32) -> bool {
            year % 4 == 0
        }

        pub(crate) const fn fixed_from_julian(date: ArithmeticDate<Julian>) -> i32 {
            let year = if date.year < 0 {
                date.year + 1
            } else {
                date.year
            };
            let mut fixed: i32 = JULIAN_EPOCH - 1 + 365 * (year - 1) + quotient(year - 1, 4);
            fixed += quotient(367 * (date.month as i32) - 362, 12);
            fixed += if date.month <= 2 {
                0
            } else if Self::is_leap_year_const(date.year) {
                -1
            } else {
                -2
            };

            fixed + (date.day as i32)
        }

        pub(crate) const fn fixed_from_julian_integers(year: i32, month: u8, day: u8) -> i32 {
            Self::fixed_from_julian(ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            })
        }

        fn days_in_year_direct(year: i32) -> u32 {
            if Julian::is_leap_year(year) {
                366
            } else {
                365
            }
        }

        fn julian_from_fixed(date: i32) -> JulianDateInner {
            let approx = quotient((4 * date) + 1464, 1461);
            let year = if approx <= 0 { approx - 1 } else { approx };
            let prior_days = date - Self::fixed_from_julian_integers(year, 1, 1);
            let correction = if date < Self::fixed_from_julian_integers(year, 3, 1) {
                0
            } else if Julian::is_leap_year(year) {
                1
            } else {
                2
            };
            let month = quotient(12 * (prior_days + correction) + 373, 367) as u8; // this expression is in 1..=12
            let day = (date - Self::fixed_from_julian_integers(year, month, 1) + 1) as u8; // as days_in_month is < u8::MAX

            #[allow(clippy::unwrap_used)] // day and month have the correct bounds
            *Date::try_new_julian_date(year, month, day).unwrap().inner()
        }
    }

    impl Date<Julian> {
        pub fn try_new_julian_date(
            year: i32,
            month: u8,
            day: u8,
        ) -> Result<Date<Julian>, CalendarError> {
            let inner = ArithmeticDate {
                year,
                month,
                day,
                marker: PhantomData,
            };

            if day > 28 {
                let bound = inner.days_in_month();
                if day > bound {
                    return Err(CalendarError::OutOfRange);
                }
            }

            Ok(Date::from_raw(JulianDateInner(inner), Julian))
        }
    }

    impl DateTime<Julian> {
        pub fn try_new_julian_datetime(
            year: i32,
            month: u8,
            day: u8,
            hour: u8,
            minute: u8,
            second: u8,
        ) -> Result<DateTime<Julian>, CalendarError> {
            Ok(DateTime {
                date: Date::try_new_julian_date(year, month, day)?,
                time: types::Time::try_new(hour, minute, second, 0)?,
            })
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_day_iso_to_julian() {
            let iso_date = Date::try_new_iso_date(200, 3, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 200);
            assert_eq!(julian_date.0.month, 3);
            assert_eq!(julian_date.0.day, 1);

            let iso_date = Date::try_new_iso_date(200, 2, 28).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 200);
            assert_eq!(julian_date.0.month, 2);
            assert_eq!(julian_date.0.day, 29);

            let iso_date = Date::try_new_iso_date(400, 3, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 400);
            assert_eq!(julian_date.0.month, 2);
            assert_eq!(julian_date.0.day, 29);

            let iso_date = Date::try_new_iso_date(2022, 1, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 2021);
            assert_eq!(julian_date.0.month, 12);
            assert_eq!(julian_date.0.day, 19);
        }

        #[test]
        fn test_day_julian_to_iso() {
            let julian_date = Date::try_new_julian_date(200, 3, 1).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(200, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);

            let julian_date = Date::try_new_julian_date(200, 2, 29).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(200, 2, 28).unwrap();
            assert_eq!(iso_date, iso_expected_date);

            let julian_date = Date::try_new_julian_date(400, 2, 29).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(400, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);

            let julian_date = Date::try_new_julian_date(2021, 12, 19).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(2022, 1, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);

            let julian_date = Date::try_new_julian_date(2022, 2, 16).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(2022, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);
        }

        #[test]
        fn test_roundtrip_negative() {
            let iso_date = Date::try_new_iso_date(-1000, 3, 3).unwrap();
            let julian = iso_date.to_calendar(Julian::new());
            let recovered_iso = julian.to_iso();
            assert_eq!(iso_date, recovered_iso);
        }
    }
}
pub mod provider {


    #![allow(clippy::exhaustive_structs, clippy::exhaustive_enums)]

    use crate::types::IsoWeekday;
    use core::str::FromStr;
    use icu_provider::{yoke, zerofrom};
    use tinystr::TinyStr16;
    use zerovec::ZeroVec;

    #[zerovec::make_ule(EraStartDateULE)]
    #[derive(
        Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash, Debug, yoke::Yokeable, zerofrom::ZeroFrom,
    )]
    #[cfg_attr(
        feature = "datagen",
        derive(serde::Serialize, databake::Bake),
        databake(path = icu_calendar::provider),
    )]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    pub struct EraStartDate {
        pub year: i32,
        pub month: u8,
        pub day: u8,
    }

    #[icu_provider::data_struct(
        marker(JapaneseErasV1Marker, "calendar/japanese@1"),
        marker(JapaneseExtendedErasV1Marker, "calendar/japanext@1")
    )]
    #[derive(Debug, PartialEq, Clone, Default)]
    #[cfg_attr(
        feature = "datagen",
        derive(serde::Serialize, databake::Bake),
        databake(path = icu_calendar::provider),
    )]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    pub struct JapaneseErasV1<'data> {
        #[cfg_attr(feature = "serde", serde(borrow))]
        pub dates_to_eras: ZeroVec<'data, (EraStartDate, TinyStr16)>,
    }

    impl FromStr for EraStartDate {
        type Err = ();
        fn from_str(mut s: &str) -> Result<Self, ()> {
            let sign = if let Some(suffix) = s.strip_prefix('-') {
                s = suffix;
                -1
            } else {
                1
            };

            let mut split = s.split('-');
            let year = split.next().ok_or(())?.parse::<i32>().map_err(|_| ())? * sign;
            let month = split.next().ok_or(())?.parse().map_err(|_| ())?;
            let day = split.next().ok_or(())?.parse().map_err(|_| ())?;

            Ok(EraStartDate { year, month, day })
        }
    }

    #[icu_provider::data_struct(marker(
        WeekDataV1Marker,
        "datetime/week_data@1",
        fallback_by = "region"
    ))]
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(
        feature = "datagen",
        derive(serde::Serialize, databake::Bake),
        databake(path = icu_calendar::provider),
    )]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    #[allow(clippy::exhaustive_structs)] // used in data provider
    pub struct WeekDataV1 {
        pub first_weekday: IsoWeekday,
        pub min_week_days: u8,
    }
}
pub mod types {


    use crate::error::CalendarError;
    use crate::helpers;
    use core::convert::TryFrom;
    use core::convert::TryInto;
    use core::fmt;
    use core::str::FromStr;
    use tinystr::{TinyStr16, TinyStr4};
    use zerovec::maps::ZeroMapKV;
    use zerovec::ule::AsULE;

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct Era(pub TinyStr16);

    impl From<TinyStr16> for Era {
        fn from(x: TinyStr16) -> Self {
            Self(x)
        }
    }

    impl FromStr for Era {
        type Err = <TinyStr16 as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse().map(Self)
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[non_exhaustive]
    pub struct FormattableYear {
        pub era: Era,

        pub number: i32,

        pub related_iso: Option<i32>,
    }

    impl FormattableYear {
        pub fn new(era: Era, number: i32) -> Self {
            Self {
                era,
                number,
                related_iso: None,
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    #[cfg_attr(
        feature = "datagen",
        derive(serde::Serialize, databake::Bake),
        databake(path = icu_calendar::types),
    )]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    pub struct MonthCode(pub TinyStr4);

    impl AsULE for MonthCode {
        type ULE = TinyStr4;
        fn to_unaligned(self) -> TinyStr4 {
            self.0
        }
        fn from_unaligned(u: TinyStr4) -> Self {
            Self(u)
        }
    }

    impl<'a> ZeroMapKV<'a> for MonthCode {
        type Container = zerovec::ZeroVec<'a, MonthCode>;
        type Slice = zerovec::ZeroSlice<MonthCode>;
        type GetType = <MonthCode as AsULE>::ULE;
        type OwnedType = MonthCode;
    }

    impl fmt::Display for MonthCode {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<TinyStr4> for MonthCode {
        fn from(x: TinyStr4) -> Self {
            Self(x)
        }
    }
    impl FromStr for MonthCode {
        type Err = <TinyStr4 as FromStr>::Err;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            s.parse().map(Self)
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct FormattableMonth {
        pub ordinal: u32,

        pub code: MonthCode,
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct DayOfYearInfo {
        pub day_of_year: u32,
        pub days_in_year: u32,
        pub prev_year: FormattableYear,
        pub days_in_prev_year: u32,
        pub next_year: FormattableYear,
    }

    #[allow(clippy::exhaustive_structs)] // this is a newtype
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DayOfMonth(pub u32);

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct WeekOfMonth(pub u32);

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct WeekOfYear(pub u32);

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct DayOfWeekInMonth(pub u32);

    impl From<DayOfMonth> for DayOfWeekInMonth {
        fn from(day_of_month: DayOfMonth) -> Self {
            DayOfWeekInMonth(1 + ((day_of_month.0 - 1) / 7))
        }
    }

    #[test]
    fn test_day_of_week_in_month() {
        assert_eq!(DayOfWeekInMonth::from(DayOfMonth(1)).0, 1);
        assert_eq!(DayOfWeekInMonth::from(DayOfMonth(7)).0, 1);
        assert_eq!(DayOfWeekInMonth::from(DayOfMonth(8)).0, 2);
    }

    macro_rules! dt_unit {
        ($name:ident, $storage:ident, $value:expr, $docs:expr) => {
            #[doc=$docs]
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
            pub struct $name($storage);

            impl $name {
                pub const fn number(self) -> $storage {
                    self.0
                }

                pub const fn zero() -> $name {
                    Self(0)
                }
            }

            impl FromStr for $name {
                type Err = CalendarError;

                fn from_str(input: &str) -> Result<Self, Self::Err> {
                    let val: $storage = input.parse()?;
                    if val > $value {
                        Err(CalendarError::Overflow {
                            field: "$name",
                            max: $value,
                        })
                    } else {
                        Ok(Self(val))
                    }
                }
            }

            impl TryFrom<$storage> for $name {
                type Error = CalendarError;

                fn try_from(input: $storage) -> Result<Self, Self::Error> {
                    if input > $value {
                        Err(CalendarError::Overflow {
                            field: "$name",
                            max: $value,
                        })
                    } else {
                        Ok(Self(input))
                    }
                }
            }

            impl TryFrom<usize> for $name {
                type Error = CalendarError;

                fn try_from(input: usize) -> Result<Self, Self::Error> {
                    if input > $value {
                        Err(CalendarError::Overflow {
                            field: "$name",
                            max: $value,
                        })
                    } else {
                        Ok(Self(input as $storage))
                    }
                }
            }

            impl From<$name> for $storage {
                fn from(input: $name) -> Self {
                    input.0
                }
            }

            impl From<$name> for usize {
                fn from(input: $name) -> Self {
                    input.0 as Self
                }
            }

            impl $name {
                pub fn try_add(self, other: $storage) -> Option<Self> {
                    let sum = self.0.saturating_add(other);
                    if sum > $value {
                        None
                    } else {
                        Some(Self(sum))
                    }
                }

                pub fn try_sub(self, other: $storage) -> Option<Self> {
                    self.0.checked_sub(other).map(Self)
                }
            }
        };
    }

    dt_unit!(
        IsoHour,
        u8,
        24,
        "An ISO-8601 hour component, for use with ISO calendars.\n\nMust be within inclusive bounds `[0, 24]`."
    );

    dt_unit!(
        IsoMinute,
        u8,
        60,
        "An ISO-8601 minute component, for use with ISO calendars.\n\nMust be within inclusive bounds `[0, 60]`."
    );

    dt_unit!(
        IsoSecond,
        u8,
        61,
        "An ISO-8601 second component, for use with ISO calendars.\n\nMust be within inclusive bounds `[0, 61]`."
    );

    dt_unit!(
        NanoSecond,
        u32,
        999_999_999,
        "A fractional second component, stored as nanoseconds.\n\nMust be within inclusive bounds `[0, 999_999_999]`."
    );

    #[test]
    fn test_iso_hour_arithmetic() {
        const HOUR_MAX: u8 = 24;
        const HOUR_VALUE: u8 = 5;
        let hour = IsoHour(HOUR_VALUE);

        assert_eq!(
            hour.try_add(HOUR_VALUE - 1),
            Some(IsoHour(HOUR_VALUE + (HOUR_VALUE - 1)))
        );
        assert_eq!(
            hour.try_sub(HOUR_VALUE - 1),
            Some(IsoHour(HOUR_VALUE - (HOUR_VALUE - 1)))
        );

        assert_eq!(hour.try_add(HOUR_MAX - HOUR_VALUE), Some(IsoHour(HOUR_MAX)));
        assert_eq!(hour.try_sub(HOUR_VALUE), Some(IsoHour(0)));

        assert_eq!(hour.try_add(1 + HOUR_MAX - HOUR_VALUE), None);
        assert_eq!(hour.try_sub(1 + HOUR_VALUE), None);
    }

    #[test]
    fn test_iso_minute_arithmetic() {
        const MINUTE_MAX: u8 = 60;
        const MINUTE_VALUE: u8 = 5;
        let minute = IsoMinute(MINUTE_VALUE);

        assert_eq!(
            minute.try_add(MINUTE_VALUE - 1),
            Some(IsoMinute(MINUTE_VALUE + (MINUTE_VALUE - 1)))
        );
        assert_eq!(
            minute.try_sub(MINUTE_VALUE - 1),
            Some(IsoMinute(MINUTE_VALUE - (MINUTE_VALUE - 1)))
        );

        assert_eq!(
            minute.try_add(MINUTE_MAX - MINUTE_VALUE),
            Some(IsoMinute(MINUTE_MAX))
        );
        assert_eq!(minute.try_sub(MINUTE_VALUE), Some(IsoMinute(0)));

        assert_eq!(minute.try_add(1 + MINUTE_MAX - MINUTE_VALUE), None);
        assert_eq!(minute.try_sub(1 + MINUTE_VALUE), None);
    }

    #[test]
    fn test_iso_second_arithmetic() {
        const SECOND_MAX: u8 = 61;
        const SECOND_VALUE: u8 = 5;
        let second = IsoSecond(SECOND_VALUE);

        assert_eq!(
            second.try_add(SECOND_VALUE - 1),
            Some(IsoSecond(SECOND_VALUE + (SECOND_VALUE - 1)))
        );
        assert_eq!(
            second.try_sub(SECOND_VALUE - 1),
            Some(IsoSecond(SECOND_VALUE - (SECOND_VALUE - 1)))
        );

        assert_eq!(
            second.try_add(SECOND_MAX - SECOND_VALUE),
            Some(IsoSecond(SECOND_MAX))
        );
        assert_eq!(second.try_sub(SECOND_VALUE), Some(IsoSecond(0)));

        assert_eq!(second.try_add(1 + SECOND_MAX - SECOND_VALUE), None);
        assert_eq!(second.try_sub(1 + SECOND_VALUE), None);
    }

    #[test]
    fn test_iso_nano_second_arithmetic() {
        const NANO_SECOND_MAX: u32 = 999_999_999;
        const NANO_SECOND_VALUE: u32 = 5;
        let nano_second = NanoSecond(NANO_SECOND_VALUE);

        assert_eq!(
            nano_second.try_add(NANO_SECOND_VALUE - 1),
            Some(NanoSecond(NANO_SECOND_VALUE + (NANO_SECOND_VALUE - 1)))
        );
        assert_eq!(
            nano_second.try_sub(NANO_SECOND_VALUE - 1),
            Some(NanoSecond(NANO_SECOND_VALUE - (NANO_SECOND_VALUE - 1)))
        );

        assert_eq!(
            nano_second.try_add(NANO_SECOND_MAX - NANO_SECOND_VALUE),
            Some(NanoSecond(NANO_SECOND_MAX))
        );
        assert_eq!(nano_second.try_sub(NANO_SECOND_VALUE), Some(NanoSecond(0)));

        assert_eq!(
            nano_second.try_add(1 + NANO_SECOND_MAX - NANO_SECOND_VALUE),
            None
        );
        assert_eq!(nano_second.try_sub(1 + NANO_SECOND_VALUE), None);
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Time {
        pub hour: IsoHour,

        pub minute: IsoMinute,

        pub second: IsoSecond,

        pub nanosecond: NanoSecond,
    }

    impl Time {
        pub const fn new(
            hour: IsoHour,
            minute: IsoMinute,
            second: IsoSecond,
            nanosecond: NanoSecond,
        ) -> Self {
            Self {
                hour,
                minute,
                second,
                nanosecond,
            }
        }

        pub fn try_new(
            hour: u8,
            minute: u8,
            second: u8,
            nanosecond: u32,
        ) -> Result<Self, CalendarError> {
            Ok(Self {
                hour: hour.try_into()?,
                minute: minute.try_into()?,
                second: second.try_into()?,
                nanosecond: nanosecond.try_into()?,
            })
        }

        pub(crate) fn from_minute_with_remainder_days(minute: i32) -> (Time, i32) {
            let (extra_days, minute_in_day) = helpers::div_rem_euclid(minute, 1440);
            let (hours, minutes) = (minute_in_day / 60, minute_in_day % 60);
            #[allow(clippy::unwrap_used)] // values are moduloed to be in range
            (
                Self {
                    hour: (hours as u8).try_into().unwrap(),
                    minute: (minutes as u8).try_into().unwrap(),
                    second: IsoSecond::zero(),
                    nanosecond: NanoSecond::zero(),
                },
                extra_days,
            )
        }
    }

    #[test]
    fn test_from_minute_with_remainder_days() {
        #[derive(Debug)]
        struct TestCase {
            minute: i32,
            expected_time: Time,
            expected_remainder: i32,
        }
        let zero_time = Time::new(
            IsoHour::zero(),
            IsoMinute::zero(),
            IsoSecond::zero(),
            NanoSecond::zero(),
        );
        let first_minute_in_day = Time::new(
            IsoHour::zero(),
            IsoMinute::try_from(1u8).unwrap(),
            IsoSecond::zero(),
            NanoSecond::zero(),
        );
        let last_minute_in_day = Time::new(
            IsoHour::try_from(23u8).unwrap(),
            IsoMinute::try_from(59u8).unwrap(),
            IsoSecond::zero(),
            NanoSecond::zero(),
        );
        let cases = [
            TestCase {
                minute: 0,
                expected_time: zero_time,
                expected_remainder: 0,
            },
            TestCase {
                minute: 30,
                expected_time: Time::new(
                    IsoHour::zero(),
                    IsoMinute::try_from(30u8).unwrap(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: 0,
            },
            TestCase {
                minute: 60,
                expected_time: Time::new(
                    IsoHour::try_from(1u8).unwrap(),
                    IsoMinute::zero(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: 0,
            },
            TestCase {
                minute: 90,
                expected_time: Time::new(
                    IsoHour::try_from(1u8).unwrap(),
                    IsoMinute::try_from(30u8).unwrap(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: 0,
            },
            TestCase {
                minute: 1439,
                expected_time: last_minute_in_day,
                expected_remainder: 0,
            },
            TestCase {
                minute: 1440,
                expected_time: Time::new(
                    IsoHour::zero(),
                    IsoMinute::zero(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: 1,
            },
            TestCase {
                minute: 1441,
                expected_time: first_minute_in_day,
                expected_remainder: 1,
            },
            TestCase {
                minute: i32::MAX,
                expected_time: Time::new(
                    IsoHour::try_from(2u8).unwrap(),
                    IsoMinute::try_from(7u8).unwrap(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: 1491308,
            },
            TestCase {
                minute: -1,
                expected_time: last_minute_in_day,
                expected_remainder: -1,
            },
            TestCase {
                minute: -1439,
                expected_time: first_minute_in_day,
                expected_remainder: -1,
            },
            TestCase {
                minute: -1440,
                expected_time: zero_time,
                expected_remainder: -1,
            },
            TestCase {
                minute: -1441,
                expected_time: last_minute_in_day,
                expected_remainder: -2,
            },
            TestCase {
                minute: i32::MIN,
                expected_time: Time::new(
                    IsoHour::try_from(21u8).unwrap(),
                    IsoMinute::try_from(52u8).unwrap(),
                    IsoSecond::zero(),
                    NanoSecond::zero(),
                ),
                expected_remainder: -1491309,
            },
        ];
        for cas in cases {
            let (actual_time, actual_remainder) = Time::from_minute_with_remainder_days(cas.minute);
            assert_eq!(actual_time, cas.expected_time, "{cas:?}");
            assert_eq!(actual_remainder, cas.expected_remainder, "{cas:?}");
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[allow(missing_docs)] // The weekday variants should be self-obvious.
    #[repr(i8)]
    #[cfg_attr(
        feature = "datagen",
        derive(serde::Serialize, databake::Bake),
        databake(path = icu_calendar::types),
    )]
    #[cfg_attr(feature = "serde", derive(serde::Deserialize))]
    #[allow(clippy::exhaustive_enums)] // This is stable
    pub enum IsoWeekday {
        Monday = 1,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
        Sunday,
    }

    impl From<usize> for IsoWeekday {
        fn from(input: usize) -> Self {
            let mut ordinal = (input % 7) as i8;
            if ordinal == 0 {
                ordinal = 7;
            }
            unsafe { core::mem::transmute(ordinal) }
        }
    }
}
mod week_of {

    use crate::{
        error::CalendarError,
        provider::WeekDataV1,
        types::{DayOfMonth, DayOfYearInfo, IsoWeekday, WeekOfMonth},
    };
    use icu_provider::prelude::*;

    pub const MIN_UNIT_DAYS: u16 = 14;

    #[derive(Clone, Copy, Debug)]
    #[non_exhaustive]
    pub struct WeekCalculator {
        pub first_weekday: IsoWeekday,
        pub min_week_days: u8,
    }

    impl From<WeekDataV1> for WeekCalculator {
        fn from(other: WeekDataV1) -> Self {
            Self {
                first_weekday: other.first_weekday,
                min_week_days: other.min_week_days,
            }
        }
    }

    impl From<&WeekDataV1> for WeekCalculator {
        fn from(other: &WeekDataV1) -> Self {
            Self {
                first_weekday: other.first_weekday,
                min_week_days: other.min_week_days,
            }
        }
    }

    impl WeekCalculator {
        pub fn try_new_unstable<P>(provider: &P, locale: &DataLocale) -> Result<Self, CalendarError>
        where
            P: DataProvider<crate::provider::WeekDataV1Marker>,
        {
            provider
                .load(DataRequest {
                    locale,
                    metadata: Default::default(),
                })
                .and_then(DataResponse::take_payload)
                .map(|payload| payload.get().into())
                .map_err(Into::into)
        }

        icu_provider::gen_any_buffer_constructors!(
            locale: include,
            options: skip,
            error: CalendarError
        );

        pub fn week_of_month(
            &self,
            day_of_month: DayOfMonth,
            iso_weekday: IsoWeekday,
        ) -> WeekOfMonth {
            WeekOfMonth(
                simple_week_of(self.first_weekday, day_of_month.0 as u16, iso_weekday) as u32,
            )
        }

        pub fn week_of_year(
            &self,
            day_of_year_info: DayOfYearInfo,
            iso_weekday: IsoWeekday,
        ) -> Result<WeekOf, CalendarError> {
            week_of(
                self,
                day_of_year_info.days_in_prev_year as u16,
                day_of_year_info.days_in_year as u16,
                day_of_year_info.day_of_year as u16,
                iso_weekday,
            )
        }

        fn weekday_index(&self, weekday: IsoWeekday) -> i8 {
            (7 + (weekday as i8) - (self.first_weekday as i8)) % 7
        }
    }

    impl Default for WeekCalculator {
        fn default() -> Self {
            Self {
                first_weekday: IsoWeekday::Monday,
                min_week_days: 1,
            }
        }
    }

    fn add_to_weekday(weekday: IsoWeekday, num_days: i32) -> IsoWeekday {
        let new_weekday = (7 + (weekday as i32) + (num_days % 7)) % 7;
        IsoWeekday::from(new_weekday as usize)
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::enum_variant_names)]
    enum RelativeWeek {
        LastWeekOfPreviousUnit,
        WeekOfCurrentUnit(u16),
        FirstWeekOfNextUnit,
    }

    struct UnitInfo {
        first_day: IsoWeekday,
        duration_days: u16,
    }

    impl UnitInfo {
        fn new(first_day: IsoWeekday, duration_days: u16) -> Result<UnitInfo, CalendarError> {
            if duration_days < MIN_UNIT_DAYS {
                return Err(CalendarError::Underflow {
                    field: "Month/Year duration",
                    min: MIN_UNIT_DAYS as isize,
                });
            }
            Ok(UnitInfo {
                first_day,
                duration_days,
            })
        }

        fn first_week_offset(&self, calendar: &WeekCalculator) -> i8 {
            let first_day_index = calendar.weekday_index(self.first_day);
            if 7 - first_day_index >= calendar.min_week_days as i8 {
                -first_day_index
            } else {
                7 - first_day_index
            }
        }

        fn num_weeks(&self, calendar: &WeekCalculator) -> u16 {
            let first_week_offset = self.first_week_offset(calendar);
            let num_days_including_first_week =
                (self.duration_days as i32) - (first_week_offset as i32);
            debug_assert!(
                num_days_including_first_week >= 0,
                "Unit is shorter than a week."
            );
            ((num_days_including_first_week + 7 - (calendar.min_week_days as i32)) / 7) as u16
        }

        fn relative_week(&self, calendar: &WeekCalculator, day: u16) -> RelativeWeek {
            let days_since_first_week =
                i32::from(day) - i32::from(self.first_week_offset(calendar)) - 1;
            if days_since_first_week < 0 {
                return RelativeWeek::LastWeekOfPreviousUnit;
            }

            let week_number = (1 + days_since_first_week / 7) as u16;
            if week_number > self.num_weeks(calendar) {
                return RelativeWeek::FirstWeekOfNextUnit;
            }
            RelativeWeek::WeekOfCurrentUnit(week_number)
        }
    }

    #[derive(Debug, PartialEq)]
    #[allow(clippy::exhaustive_enums)] // this type is stable
    pub enum RelativeUnit {
        Previous,
        Current,
        Next,
    }

    #[derive(Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct WeekOf {
        pub week: u16,
        pub unit: RelativeUnit,
    }

    pub fn week_of(
        calendar: &WeekCalculator,
        num_days_in_previous_unit: u16,
        num_days_in_unit: u16,
        day: u16,
        week_day: IsoWeekday,
    ) -> Result<WeekOf, CalendarError> {
        let current = UnitInfo::new(
            add_to_weekday(week_day, 1 - i32::from(day)),
            num_days_in_unit,
        )?;

        match current.relative_week(calendar, day) {
            RelativeWeek::LastWeekOfPreviousUnit => {
                let previous = UnitInfo::new(
                    add_to_weekday(current.first_day, -i32::from(num_days_in_previous_unit)),
                    num_days_in_previous_unit,
                )?;

                Ok(WeekOf {
                    week: previous.num_weeks(calendar),
                    unit: RelativeUnit::Previous,
                })
            }
            RelativeWeek::WeekOfCurrentUnit(w) => Ok(WeekOf {
                week: w,
                unit: RelativeUnit::Current,
            }),
            RelativeWeek::FirstWeekOfNextUnit => Ok(WeekOf {
                week: 1,
                unit: RelativeUnit::Next,
            }),
        }
    }

    pub fn simple_week_of(first_weekday: IsoWeekday, day: u16, week_day: IsoWeekday) -> u16 {
        let calendar = WeekCalculator {
            first_weekday,
            min_week_days: 1,
        };

        #[allow(clippy::unwrap_used)] // week_of should can't fail with MIN_UNIT_DAYS
        week_of(
            &calendar,
            MIN_UNIT_DAYS,
            u16::MAX,
            day,
            week_day,
        )
        .unwrap()
        .week
    }

    #[cfg(test)]
    mod tests {
        use super::{week_of, RelativeUnit, RelativeWeek, UnitInfo, WeekCalculator, WeekOf};
        use crate::{error::CalendarError, types::IsoWeekday, Date, DateDuration};

        static ISO_CALENDAR: WeekCalculator = WeekCalculator {
            first_weekday: IsoWeekday::Monday,
            min_week_days: 4,
        };

        static AE_CALENDAR: WeekCalculator = WeekCalculator {
            first_weekday: IsoWeekday::Saturday,
            min_week_days: 4,
        };

        static US_CALENDAR: WeekCalculator = WeekCalculator {
            first_weekday: IsoWeekday::Sunday,
            min_week_days: 1,
        };

        #[test]
        fn test_weekday_index() {
            assert_eq!(ISO_CALENDAR.weekday_index(IsoWeekday::Monday), 0);
            assert_eq!(ISO_CALENDAR.weekday_index(IsoWeekday::Sunday), 6);

            assert_eq!(AE_CALENDAR.weekday_index(IsoWeekday::Saturday), 0);
            assert_eq!(AE_CALENDAR.weekday_index(IsoWeekday::Friday), 6);
        }

        #[test]
        fn test_first_week_offset() {
            let first_week_offset =
                |calendar, day| UnitInfo::new(day, 30).unwrap().first_week_offset(calendar);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Monday), 0);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Tuesday), -1);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Wednesday), -2);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Thursday), -3);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Friday), 3);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Saturday), 2);
            assert_eq!(first_week_offset(&ISO_CALENDAR, IsoWeekday::Sunday), 1);

            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Saturday), 0);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Sunday), -1);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Monday), -2);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Tuesday), -3);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Wednesday), 3);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Thursday), 2);
            assert_eq!(first_week_offset(&AE_CALENDAR, IsoWeekday::Friday), 1);

            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Sunday), 0);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Monday), -1);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Tuesday), -2);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Wednesday), -3);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Thursday), -4);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Friday), -5);
            assert_eq!(first_week_offset(&US_CALENDAR, IsoWeekday::Saturday), -6);
        }

        #[test]
        fn test_num_weeks() -> Result<(), CalendarError> {
            assert_eq!(
                UnitInfo::new(IsoWeekday::Thursday, 4 + 2 * 7 + 4)?.num_weeks(&ISO_CALENDAR),
                4
            );
            assert_eq!(
                UnitInfo::new(IsoWeekday::Friday, 3 + 2 * 7 + 4)?.num_weeks(&ISO_CALENDAR),
                3
            );
            assert_eq!(
                UnitInfo::new(IsoWeekday::Friday, 3 + 2 * 7 + 3)?.num_weeks(&ISO_CALENDAR),
                2
            );

            assert_eq!(
                UnitInfo::new(IsoWeekday::Saturday, 1 + 2 * 7 + 1)?.num_weeks(&US_CALENDAR),
                4
            );
            Ok(())
        }

        fn classify_days_of_unit(calendar: &WeekCalculator, unit: &UnitInfo) -> Vec<RelativeWeek> {
            let mut weeks: Vec<Vec<IsoWeekday>> = Vec::new();
            for day_index in 0..unit.duration_days {
                let day = super::add_to_weekday(unit.first_day, i32::from(day_index));
                if day == calendar.first_weekday || weeks.is_empty() {
                    weeks.push(Vec::new());
                }
                weeks.last_mut().unwrap().push(day);
            }

            let mut day_week_of_units = Vec::new();
            let mut weeks_in_unit = 0;
            for (index, week) in weeks.iter().enumerate() {
                let week_of_unit = if week.len() < usize::from(calendar.min_week_days) {
                    match index {
                        0 => RelativeWeek::LastWeekOfPreviousUnit,
                        x if x == weeks.len() - 1 => RelativeWeek::FirstWeekOfNextUnit,
                        _ => panic!(),
                    }
                } else {
                    weeks_in_unit += 1;
                    RelativeWeek::WeekOfCurrentUnit(weeks_in_unit)
                };

                day_week_of_units.append(&mut [week_of_unit].repeat(week.len()));
            }
            day_week_of_units
        }

        #[test]
        fn test_relative_week_of_month() -> Result<(), CalendarError> {
            for min_week_days in 1..7 {
                for start_of_week in 1..7 {
                    let calendar = WeekCalculator {
                        first_weekday: IsoWeekday::from(start_of_week),
                        min_week_days,
                    };
                    for unit_duration in super::MIN_UNIT_DAYS..400 {
                        for start_of_unit in 1..7 {
                            let unit =
                                UnitInfo::new(IsoWeekday::from(start_of_unit), unit_duration)?;
                            let expected = classify_days_of_unit(&calendar, &unit);
                            for (index, expected_week_of) in expected.iter().enumerate() {
                                let day = index + 1;
                                assert_eq!(
                                    unit.relative_week(&calendar, day as u16),
                                    *expected_week_of,
                                    "For the {day}/{unit_duration} starting on IsoWeekday \
                            {start_of_unit} using start_of_week {start_of_week} \
                            & min_week_days {min_week_days}"
                                );
                            }
                        }
                    }
                }
            }
            Ok(())
        }

        fn week_of_month_from_iso_date(
            calendar: &WeekCalculator,
            yyyymmdd: u32,
        ) -> Result<WeekOf, CalendarError> {
            let year = (yyyymmdd / 10000) as i32;
            let month = ((yyyymmdd / 100) % 100) as u8;
            let day = (yyyymmdd % 100) as u8;

            let date = Date::try_new_iso_date(year, month, day)?;
            let previous_month = date.added(DateDuration::new(0, -1, 0, 0));

            week_of(
                calendar,
                u16::from(previous_month.days_in_month()),
                u16::from(date.days_in_month()),
                u16::from(day),
                date.day_of_week(),
            )
        }

        #[test]
        fn test_week_of_month_using_dates() -> Result<(), CalendarError> {
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20210418)?,
                WeekOf {
                    week: 3,
                    unit: RelativeUnit::Current,
                }
            );
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20210419)?,
                WeekOf {
                    week: 4,
                    unit: RelativeUnit::Current,
                }
            );

            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20180101)?,
                WeekOf {
                    week: 1,
                    unit: RelativeUnit::Current,
                }
            );
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20210101)?,
                WeekOf {
                    week: 5,
                    unit: RelativeUnit::Previous,
                }
            );

            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20200930)?,
                WeekOf {
                    week: 1,
                    unit: RelativeUnit::Next,
                }
            );
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20201231)?,
                WeekOf {
                    week: 5,
                    unit: RelativeUnit::Current,
                }
            );

            assert_eq!(
                week_of_month_from_iso_date(&US_CALENDAR, 20201231)?,
                WeekOf {
                    week: 5,
                    unit: RelativeUnit::Current,
                }
            );
            assert_eq!(
                week_of_month_from_iso_date(&US_CALENDAR, 20210101)?,
                WeekOf {
                    week: 1,
                    unit: RelativeUnit::Current,
                }
            );

            Ok(())
        }
    }

    #[test]
    fn test_simple_week_of() {
        assert_eq!(
            simple_week_of(IsoWeekday::Monday, 2, IsoWeekday::Tuesday),
            1
        );
        assert_eq!(simple_week_of(IsoWeekday::Monday, 7, IsoWeekday::Sunday), 1);
        assert_eq!(simple_week_of(IsoWeekday::Monday, 8, IsoWeekday::Monday), 2);

        assert_eq!(
            simple_week_of(IsoWeekday::Tuesday, 1, IsoWeekday::Wednesday),
            1
        );
        assert_eq!(
            simple_week_of(IsoWeekday::Tuesday, 6, IsoWeekday::Monday),
            1
        );
        assert_eq!(
            simple_week_of(IsoWeekday::Tuesday, 7, IsoWeekday::Tuesday),
            2
        );

        assert_eq!(
            simple_week_of(IsoWeekday::Sunday, 26, IsoWeekday::Friday),
            4
        );
    }
}

pub mod week {
    use crate::week_of;
    pub use week_of::RelativeUnit;
    pub use week_of::WeekCalculator;
    pub use week_of::WeekOf;
}

pub use any_calendar::{AnyCalendar, AnyCalendarKind};
pub use calendar::Calendar;
pub use date::{AsCalendar, Date, Ref};
pub use datetime::DateTime;
#[doc(hidden)]
pub use duration::{DateDuration, DateDurationUnit};
pub use error::CalendarError;
pub use gregorian::Gregorian;
pub use iso::Iso;

#[doc(no_inline)]
pub use CalendarError as Error;
