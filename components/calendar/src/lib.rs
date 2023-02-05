
#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

// Make sure inherent docs go first
mod date { 
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::any_calendar::{AnyCalendar, IntoAnyCalendar};
use crate::week::{WeekCalculator, WeekOf};
use crate::{types, Calendar, CalendarError, DateDuration, DateDurationUnit, Iso};
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::fmt;
use core::ops::Deref;

/// Types that contain a calendar
///
/// This allows one to use [`Date`] with wrappers around calendars,
/// e.g. reference counted calendars.
pub trait AsCalendar {
    /// The calendar being wrapped
    type Calendar: Calendar;
    /// Obtain the inner calendar
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

/// This exists as a wrapper around `&'a T` so that
/// `Date<&'a C>` is possible for calendar `C`.
///
/// Unfortunately,
/// [`AsCalendar`] cannot be implemented on `&'a T` directly because
/// `&'a T` is `#[fundamental]` and the impl would clash with the one above with
/// `AsCalendar` for `C: Calendar`.
///
/// Use `Date<Ref<'a, C>>` where you would use `Date<&'a C>`
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

/// A date for a given calendar.
///
/// This can work with wrappers around [`Calendar`] types,
/// e.g. `Rc<C>`, via the [`AsCalendar`] trait.
///
/// This can be constructed  constructed
/// from its fields via [`Self::try_new_from_codes()`], or can be constructed with one of the
/// `new_<calendar>_datetime()` per-calendar methods (and then freely converted between calendars).
///
/// ```rust
/// use icu::calendar::Date;
///
/// // Example: creation of ISO date from integers.
/// let date_iso = Date::try_new_iso_date(1970, 1, 2)
///     .expect("Failed to initialize ISO Date instance.");
///
/// assert_eq!(date_iso.year().number, 1970);
/// assert_eq!(date_iso.month().ordinal, 1);
/// assert_eq!(date_iso.day_of_month().0, 2);
/// ```
#[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
pub struct Date<A: AsCalendar> {
    pub(crate) inner: <A::Calendar as Calendar>::DateInner,
    pub(crate) calendar: A,
}

impl<A: AsCalendar> Date<A> {
    /// Construct a date from from era/month codes and fields, and some calendar representation
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

    /// Construct a date from an ISO date and some calendar representation
    #[inline]
    pub fn new_from_iso(iso: Date<Iso>, calendar: A) -> Self {
        let inner = calendar.as_calendar().date_from_iso(iso);
        Date { inner, calendar }
    }

    /// Convert the Date to an ISO Date
    #[inline]
    pub fn to_iso(&self) -> Date<Iso> {
        self.calendar.as_calendar().date_to_iso(self.inner())
    }

    /// Convert the Date to a date in a different calendar
    #[inline]
    pub fn to_calendar<A2: AsCalendar>(&self, calendar: A2) -> Date<A2> {
        Date::new_from_iso(self.to_iso(), calendar)
    }

    /// The number of months in the year of this date
    #[inline]
    pub fn months_in_year(&self) -> u8 {
        self.calendar.as_calendar().months_in_year(self.inner())
    }

    /// The number of days in the year of this date
    #[inline]
    pub fn days_in_year(&self) -> u32 {
        self.calendar.as_calendar().days_in_year(self.inner())
    }

    /// The number of days in the month of this date
    #[inline]
    pub fn days_in_month(&self) -> u8 {
        self.calendar.as_calendar().days_in_month(self.inner())
    }

    /// The day of the week for this date
    ///
    /// Monday is 1, Sunday is 7, according to ISO
    #[inline]
    pub fn day_of_week(&self) -> types::IsoWeekday {
        self.calendar.as_calendar().day_of_week(self.inner())
    }

    /// Add a `duration` to this date, mutating it
    ///
    /// Currently unstable for ICU4X 1.0
    #[doc(hidden)]
    #[inline]
    pub fn add(&mut self, duration: DateDuration<A::Calendar>) {
        self.calendar
            .as_calendar()
            .offset_date(&mut self.inner, duration)
    }

    /// Add a `duration` to this date, returning the new one
    ///
    /// Currently unstable for ICU4X 1.0
    #[doc(hidden)]
    #[inline]
    pub fn added(mut self, duration: DateDuration<A::Calendar>) -> Self {
        self.add(duration);
        self
    }

    /// Calculating the duration between `other - self`
    ///
    /// Currently unstable for ICU4X 1.0
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

    /// The calendar-specific year represented by `self`
    #[inline]
    pub fn year(&self) -> types::FormattableYear {
        self.calendar.as_calendar().year(&self.inner)
    }

    /// The calendar-specific month represented by `self`
    #[inline]
    pub fn month(&self) -> types::FormattableMonth {
        self.calendar.as_calendar().month(&self.inner)
    }

    /// The calendar-specific day-of-month represented by `self`
    #[inline]
    pub fn day_of_month(&self) -> types::DayOfMonth {
        self.calendar.as_calendar().day_of_month(&self.inner)
    }

    /// The calendar-specific day-of-month represented by `self`
    #[inline]
    pub fn day_of_year_info(&self) -> types::DayOfYearInfo {
        self.calendar.as_calendar().day_of_year_info(&self.inner)
    }

    /// The week of the month containing this date.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::calendar::types::IsoWeekday;
    /// use icu::calendar::types::WeekOfMonth;
    /// use icu::calendar::Date;
    ///
    /// let date = Date::try_new_iso_date(2022, 8, 10).unwrap(); // second Wednesday
    ///
    /// // The following info is usually locale-specific
    /// let first_weekday = IsoWeekday::Sunday;
    ///
    /// assert_eq!(date.week_of_month(first_weekday), WeekOfMonth(2));
    /// ```
    pub fn week_of_month(&self, first_weekday: types::IsoWeekday) -> types::WeekOfMonth {
        let config = WeekCalculator {
            first_weekday,
            min_week_days: 0, // ignored
        };
        config.week_of_month(self.day_of_month(), self.day_of_week())
    }

    /// The week of the year containing this date.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::calendar::types::IsoWeekday;
    /// use icu::calendar::week::RelativeUnit;
    /// use icu::calendar::week::WeekCalculator;
    /// use icu::calendar::week::WeekOf;
    /// use icu::calendar::Date;
    ///
    /// let date = Date::try_new_iso_date(2022, 8, 26).unwrap();
    ///
    /// // The following info is usually locale-specific
    /// let week_calculator = WeekCalculator::default();
    ///
    /// assert_eq!(
    ///     date.week_of_year(&week_calculator),
    ///     Ok(WeekOf {
    ///         week: 35,
    ///         unit: RelativeUnit::Current
    ///     })
    /// );
    /// ```
    pub fn week_of_year(&self, config: &WeekCalculator) -> Result<WeekOf, CalendarError> {
        config.week_of_year(self.day_of_year_info(), self.day_of_week())
    }

    /// Construct a date from raw values for a given calendar. This does not check any
    /// invariants for the date and calendar, and should only be called by calendar implementations.
    ///
    /// Calling this outside of calendar implementations is sound, but calendar implementations are not
    /// expected to do anything sensible with such invalid dates.
    ///
    /// AnyCalendar *will* panic if AnyCalendar [`Date`] objects with mismatching
    /// date and calendar types are constructed
    #[inline]
    pub fn from_raw(inner: <A::Calendar as Calendar>::DateInner, calendar: A) -> Self {
        Self { inner, calendar }
    }

    /// Get the inner date implementation. Should not be called outside of calendar implementations
    #[inline]
    pub fn inner(&self) -> &<A::Calendar as Calendar>::DateInner {
        &self.inner
    }

    /// Get a reference to the contained calendar
    #[inline]
    pub fn calendar(&self) -> &A::Calendar {
        self.calendar.as_calendar()
    }

    /// Get a reference to the contained calendar wrapper
    ///
    /// (Useful in case the user wishes to e.g. clone an Rc)
    #[inline]
    pub fn calendar_wrapper(&self) -> &A {
        &self.calendar
    }
}

impl<C: IntoAnyCalendar, A: AsCalendar<Calendar = C>> Date<A> {
    /// Type-erase the date, converting it to a date for [`AnyCalendar`]
    pub fn to_any(&self) -> Date<AnyCalendar> {
        let cal = self.calendar();
        Date::from_raw(cal.date_to_any(self.inner()), cal.to_any_cloned())
    }
}

impl<C: Calendar> Date<C> {
    /// Wrap the calendar type in `Rc<T>`
    ///
    /// Useful when paired with [`Self::to_any()`] to obtain a `Date<Rc<AnyCalendar>>`
    pub fn wrap_calendar_in_rc(self) -> Date<Rc<C>> {
        Date::from_raw(self.inner, Rc::new(self.calendar))
    }

    /// Wrap the calendar type in `Arc<T>`
    ///
    /// Useful when paired with [`Self::to_any()`] to obtain a `Date<Rc<AnyCalendar>>`
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
mod datetime{
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::any_calendar::{AnyCalendar, IntoAnyCalendar};
use crate::types::{self, Time};
use crate::{AsCalendar, Calendar, CalendarError, Date, Iso};
use alloc::rc::Rc;
use alloc::sync::Arc;

/// A date+time for a given calendar.
///
/// This can work with wrappers around [`Calendar`](crate::Calendar) types,
/// e.g. `Rc<C>`, via the [`AsCalendar`] trait, much like
/// [`Date`].
///
/// This can be constructed manually from a [`Date`] and [`Time`], or can be constructed
/// from its fields via [`Self::try_new_from_codes()`], or can be constructed with one of the
/// `new_<calendar>_datetime()` per-calendar methods (and then freely converted between calendars).
///
/// ```rust
/// use icu::calendar::DateTime;
///
/// // Example: Construction of ISO datetime from integers.
/// let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
///     .expect("Failed to initialize ISO DateTime instance.");
///
/// assert_eq!(datetime_iso.date.year().number, 1970);
/// assert_eq!(datetime_iso.date.month().ordinal, 1);
/// assert_eq!(datetime_iso.date.day_of_month().0, 2);
/// assert_eq!(datetime_iso.time.hour.number(), 13);
/// assert_eq!(datetime_iso.time.minute.number(), 1);
/// assert_eq!(datetime_iso.time.second.number(), 0);
/// ```
#[derive(Debug)]
#[allow(clippy::exhaustive_structs)] // this type is stable
pub struct DateTime<A: AsCalendar> {
    /// The date
    pub date: Date<A>,
    /// The time
    pub time: Time,
}

impl<A: AsCalendar> DateTime<A> {
    /// Construct a [`DateTime`] for a given [`Date`] and [`Time`]
    pub fn new(date: Date<A>, time: Time) -> Self {
        DateTime { date, time }
    }

    /// Construct a datetime from from era/month codes and fields,
    /// and some calendar representation
    #[inline]
    pub fn try_new_from_codes(
        era: types::Era,
        year: i32,
        month_code: types::MonthCode,
        day: u8,
        time: Time,
        calendar: A,
    ) -> Result<Self, CalendarError> {
        let date = Date::try_new_from_codes(era, year, month_code, day, calendar)?;
        Ok(DateTime { date, time })
    }

    /// Construct a DateTime from an ISO datetime and some calendar representation
    #[inline]
    pub fn new_from_iso(iso: DateTime<Iso>, calendar: A) -> Self {
        let date = Date::new_from_iso(iso.date, calendar);
        DateTime {
            date,
            time: iso.time,
        }
    }

    /// Convert the DateTime to an ISO DateTime
    #[inline]
    pub fn to_iso(&self) -> DateTime<Iso> {
        DateTime {
            date: self.date.to_iso(),
            time: self.time,
        }
    }

    /// Convert the DateTime to a DateTime in a different calendar
    #[inline]
    pub fn to_calendar<A2: AsCalendar>(&self, calendar: A2) -> DateTime<A2> {
        DateTime {
            date: self.date.to_calendar(calendar),
            time: self.time,
        }
    }
}

impl<C: IntoAnyCalendar, A: AsCalendar<Calendar = C>> DateTime<A> {
    /// Type-erase the date, converting it to a date for [`AnyCalendar`]
    pub fn to_any(&self) -> DateTime<AnyCalendar> {
        DateTime {
            date: self.date.to_any(),
            time: self.time,
        }
    }
}

impl<C: Calendar> DateTime<C> {
    /// Wrap the calendar type in `Rc<T>`
    ///
    /// Useful when paired with [`Self::to_any()`] to obtain a `DateTime<Rc<AnyCalendar>>`
    pub fn wrap_calendar_in_rc(self) -> DateTime<Rc<C>> {
        DateTime {
            date: self.date.wrap_calendar_in_rc(),
            time: self.time,
        }
    }

    /// Wrap the calendar type in `Arc<T>`
    ///
    /// Useful when paired with [`Self::to_any()`] to obtain a `DateTime<Rc<AnyCalendar>>`
    pub fn wrap_calendar_in_arc(self) -> DateTime<Arc<C>> {
        DateTime {
            date: self.date.wrap_calendar_in_arc(),
            time: self.time,
        }
    }
}

impl<C, A, B> PartialEq<DateTime<B>> for DateTime<A>
where
    C: Calendar,
    A: AsCalendar<Calendar = C>,
    B: AsCalendar<Calendar = C>,
{
    fn eq(&self, other: &DateTime<B>) -> bool {
        self.date == other.date && self.time == other.time
    }
}

// We can do this since DateInner is required to be Eq by the Calendar trait
impl<A: AsCalendar> Eq for DateTime<A> {}

impl<A: AsCalendar + Clone> Clone for DateTime<A> {
    fn clone(&self) -> Self {
        Self {
            date: self.date.clone(),
            time: self.time,
        }
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Module for working with multiple calendars at once

use crate::buddhist::Buddhist;
use crate::coptic::Coptic;
use crate::ethiopian::{Ethiopian, EthiopianEraStyle};
use crate::gregorian::Gregorian;
use crate::indian::Indian;
use crate::iso::Iso;
use crate::japanese::{Japanese, JapaneseExtended};
use crate::{
    types, AsCalendar, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime, Ref,
};

use icu_locid::{
    extensions::unicode::Value, extensions_unicode_key as key, extensions_unicode_value as value,
    subtags_language as language, Locale,
};
use icu_provider::prelude::*;

use core::fmt;

/// This is a calendar that encompasses all formattable calendars supported by this crate
///
/// This allows for the construction of [`Date`] objects that have their calendar known at runtime.
///
/// This can be constructed by calling `.into()` on a concrete calendar type if the calendar type is known at
/// compile time. When the type is known at runtime, the [`AnyCalendar::try_new_with_any_provider()`],
/// [`AnyCalendar::try_new_with_buffer_provider()`], and [`AnyCalendar::try_new_unstable()`] methods may be used.
///
/// [`Date`](crate::Date) can also be converted to [`AnyCalendar`]-compatible ones
/// via [`Date::to_any()`](crate::Date::to_any()).
///
/// There are many ways of constructing an AnyCalendar'd date:
/// ```
/// use icu::calendar::{AnyCalendar, AnyCalendarKind, DateTime, japanese::Japanese, types::Time};
/// use icu::locid::locale;
/// # use std::str::FromStr;
/// # use std::rc::Rc;
/// # use std::convert::TryInto;
///
/// let locale = locale!("en-u-ca-japanese"); // English with the Japanese calendar
///
/// let calendar = AnyCalendar::try_new_for_locale_unstable(&icu_testdata::unstable(), &locale.into())
///                    .expect("constructing AnyCalendar failed");
/// let calendar = Rc::new(calendar); // Avoid cloning it each time
///                                   // If everything is a local reference, you may use icu_calendar::Ref instead.
///
/// // manually construct a datetime in this calendar
/// let manual_time = Time::try_new(12, 33, 12, 0).expect("failed to construct Time");
/// // construct from era code, year, month code, day, time, and a calendar
/// // This is March 28, 15 Heisei
/// let manual_datetime = DateTime::try_new_from_codes("heisei".parse().unwrap(), 15, "M03".parse().unwrap(), 28,
///                                                manual_time, calendar.clone())
///                     .expect("Failed to construct DateTime manually");
///
///
/// // construct another datetime by converting from ISO
/// let iso_datetime = DateTime::try_new_iso_datetime(2020, 9, 1, 12, 34, 28)
///     .expect("Failed to construct ISO DateTime.");
/// let iso_converted = iso_datetime.to_calendar(calendar);
///
/// // Construct a datetime in the appropriate typed calendar and convert
/// let japanese_calendar = Japanese::try_new_unstable(&icu_testdata::unstable()).unwrap();
/// let japanese_datetime = DateTime::try_new_japanese_datetime("heisei".parse().unwrap(), 15, 3, 28,
///                                                         12, 33, 12, japanese_calendar).unwrap();
/// // This is a DateTime<AnyCalendar>
/// let any_japanese_datetime = japanese_datetime.to_any();
/// ```
#[non_exhaustive]
#[derive(Clone, Debug)]
#[cfg_attr(all(test, feature = "serde"), derive(bolero::generator::TypeGenerator))]
pub enum AnyCalendar {
    /// A [`Gregorian`] calendar
    Gregorian(Gregorian),
    /// A [`Buddhist`] calendar
    Buddhist(Buddhist),
    /// A [`Japanese`] calendar
    Japanese(
        #[cfg_attr(
            all(test, feature = "serde"),
            generator(bolero::generator::constant(Japanese::try_new_unstable(
                &icu_testdata::buffer().as_deserializing()
            ).unwrap()))
        )]
        Japanese,
    ),
    /// A [`JapaneseExtended`] calendar
    JapaneseExtended(
        #[cfg_attr(
            all(test, feature = "serde"),
            generator(bolero::generator::constant(JapaneseExtended::try_new_unstable(
                &icu_testdata::buffer().as_deserializing()
            ).unwrap()))
        )]
        JapaneseExtended,
    ),
    /// An [`Ethiopian`] calendar
    Ethiopian(Ethiopian),
    /// An [`Indian`] calendar
    Indian(Indian),
    /// A [`Coptic`] calendar
    Coptic(Coptic),
    /// An [`Iso`] calendar
    Iso(Iso),
}

/// The inner date type for [`AnyCalendar`]
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum AnyDateInner {
    /// A date for a [`Gregorian`] calendar
    Gregorian(<Gregorian as Calendar>::DateInner),
    /// A date for a [`Buddhist`] calendar
    Buddhist(<Buddhist as Calendar>::DateInner),
    /// A date for a [`Japanese`] calendar
    Japanese(<Japanese as Calendar>::DateInner),
    /// A date for a [`JapaneseExtended`] calendar
    JapaneseExtended(<JapaneseExtended as Calendar>::DateInner),
    /// A date for an [`Ethiopian`] calendar
    Ethiopian(<Ethiopian as Calendar>::DateInner),
    /// A date for an [`Indian`] calendar
    Indian(<Indian as Calendar>::DateInner),
    /// A date for a [`Coptic`] calendar
    Coptic(<Coptic as Calendar>::DateInner),
    /// A date for an [`Iso`] calendar
    Iso(<Iso as Calendar>::DateInner),
}

macro_rules! match_cal_and_date {
    (match ($cal:ident, $date:ident): ($cal_matched:ident, $date_matched:ident) => $e:expr) => {
        match ($cal, $date) {
            (&Self::Gregorian(ref $cal_matched), &AnyDateInner::Gregorian(ref $date_matched)) => $e,
            (&Self::Buddhist(ref $cal_matched), &AnyDateInner::Buddhist(ref $date_matched)) => $e,
            (&Self::Japanese(ref $cal_matched), &AnyDateInner::Japanese(ref $date_matched)) => $e,
            (
                &Self::JapaneseExtended(ref $cal_matched),
                &AnyDateInner::JapaneseExtended(ref $date_matched),
            ) => $e,
            (&Self::Ethiopian(ref $cal_matched), &AnyDateInner::Ethiopian(ref $date_matched)) => $e,
            (&Self::Indian(ref $cal_matched), &AnyDateInner::Indian(ref $date_matched)) => $e,
            (&Self::Coptic(ref $cal_matched), &AnyDateInner::Coptic(ref $date_matched)) => $e,
            (&Self::Iso(ref $cal_matched), &AnyDateInner::Iso(ref $date_matched)) => $e,
            _ => panic!(
                "Found AnyCalendar with mixed calendar type {} and date type {}!",
                $cal.calendar_name(),
                $date.calendar_name()
            ),
        }
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
        let ret = match *self {
            Self::Gregorian(ref c) => {
                AnyDateInner::Gregorian(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Buddhist(ref c) => {
                AnyDateInner::Buddhist(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Japanese(ref c) => {
                AnyDateInner::Japanese(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::JapaneseExtended(ref c) => {
                AnyDateInner::JapaneseExtended(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Ethiopian(ref c) => {
                AnyDateInner::Ethiopian(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Indian(ref c) => {
                AnyDateInner::Indian(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Coptic(ref c) => {
                AnyDateInner::Coptic(c.date_from_codes(era, year, month_code, day)?)
            }
            Self::Iso(ref c) => AnyDateInner::Iso(c.date_from_codes(era, year, month_code, day)?),
        };
        Ok(ret)
    }
    fn date_from_iso(&self, iso: Date<Iso>) -> AnyDateInner {
        match *self {
            Self::Gregorian(ref c) => AnyDateInner::Gregorian(c.date_from_iso(iso)),
            Self::Buddhist(ref c) => AnyDateInner::Buddhist(c.date_from_iso(iso)),
            Self::Japanese(ref c) => AnyDateInner::Japanese(c.date_from_iso(iso)),
            Self::JapaneseExtended(ref c) => AnyDateInner::JapaneseExtended(c.date_from_iso(iso)),
            Self::Ethiopian(ref c) => AnyDateInner::Ethiopian(c.date_from_iso(iso)),
            Self::Indian(ref c) => AnyDateInner::Indian(c.date_from_iso(iso)),
            Self::Coptic(ref c) => AnyDateInner::Coptic(c.date_from_iso(iso)),
            Self::Iso(ref c) => AnyDateInner::Iso(c.date_from_iso(iso)),
        }
    }

    fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
        match_cal_and_date!(match (self, date): (c, d) => c.date_to_iso(d))
    }

    fn months_in_year(&self, date: &Self::DateInner) -> u8 {
        match_cal_and_date!(match (self, date): (c, d) => c.months_in_year(d))
    }

    fn days_in_year(&self, date: &Self::DateInner) -> u32 {
        match_cal_and_date!(match (self, date): (c, d) => c.days_in_year(d))
    }

    fn days_in_month(&self, date: &Self::DateInner) -> u8 {
        match_cal_and_date!(match (self, date): (c, d) => c.days_in_month(d))
    }

    fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
        match (self, date) {
            (Self::Gregorian(c), &mut AnyDateInner::Gregorian(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Buddhist(c), &mut AnyDateInner::Buddhist(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Japanese(c), &mut AnyDateInner::Japanese(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::JapaneseExtended(c), &mut AnyDateInner::JapaneseExtended(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Ethiopian(c), &mut AnyDateInner::Ethiopian(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Indian(c), &mut AnyDateInner::Indian(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Coptic(c), &mut AnyDateInner::Coptic(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            (Self::Iso(c), &mut AnyDateInner::Iso(ref mut d)) => {
                c.offset_date(d, offset.cast_unit())
            }
            // This is only reached from misuse of from_raw, a semi-internal api
            #[allow(clippy::panic)]
            (_, d) => panic!(
                "Found AnyCalendar with mixed calendar type {} and date type {}!",
                self.calendar_name(),
                d.calendar_name()
            ),
        }
    }

    fn until(
        &self,
        date1: &Self::DateInner,
        date2: &Self::DateInner,
        calendar2: &Self,
        largest_unit: DateDurationUnit,
        smallest_unit: DateDurationUnit,
    ) -> DateDuration<Self> {
        match (self, calendar2, date1, date2) {
            (
                Self::Gregorian(c1),
                Self::Gregorian(c2),
                AnyDateInner::Gregorian(d1),
                AnyDateInner::Gregorian(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::Buddhist(c1),
                Self::Buddhist(c2),
                AnyDateInner::Buddhist(d1),
                AnyDateInner::Buddhist(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::Japanese(c1),
                Self::Japanese(c2),
                AnyDateInner::Japanese(d1),
                AnyDateInner::Japanese(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::JapaneseExtended(c1),
                Self::JapaneseExtended(c2),
                AnyDateInner::JapaneseExtended(d1),
                AnyDateInner::JapaneseExtended(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::Ethiopian(c1),
                Self::Ethiopian(c2),
                AnyDateInner::Ethiopian(d1),
                AnyDateInner::Ethiopian(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::Indian(c1),
                Self::Indian(c2),
                AnyDateInner::Indian(d1),
                AnyDateInner::Indian(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (
                Self::Coptic(c1),
                Self::Coptic(c2),
                AnyDateInner::Coptic(d1),
                AnyDateInner::Coptic(d2),
            ) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            (Self::Iso(c1), Self::Iso(c2), AnyDateInner::Iso(d1), AnyDateInner::Iso(d2)) => c1
                .until(d1, d2, c2, largest_unit, smallest_unit)
                .cast_unit(),
            _ => {
                // attempt to convert
                let iso = calendar2.date_to_iso(date2);

                match_cal_and_date!(match (self, date1):
                    (c1, d1) => {
                        let d2 = c1.date_from_iso(iso);
                        let until = c1.until(d1, &d2, c1, largest_unit, smallest_unit);
                        until.cast_unit::<AnyCalendar>()
                    }
                )
            }
        }
    }

    /// The calendar-specific year represented by `date`
    fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
        match_cal_and_date!(match (self, date): (c, d) => c.year(d))
    }

    /// The calendar-specific month represented by `date`
    fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
        match_cal_and_date!(match (self, date): (c, d) => c.month(d))
    }

    /// The calendar-specific day-of-month represented by `date`
    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        match_cal_and_date!(match (self, date): (c, d) => c.day_of_month(d))
    }

    /// Information of the day of the year
    fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
        match_cal_and_date!(match (self, date): (c, d) => c.day_of_year_info(d))
    }

    fn debug_name(&self) -> &'static str {
        match *self {
            Self::Gregorian(_) => "AnyCalendar (Gregorian)",
            Self::Buddhist(_) => "AnyCalendar (Buddhist)",
            Self::Japanese(_) => "AnyCalendar (Japanese)",
            Self::JapaneseExtended(_) => "AnyCalendar (Japanese, Historical Era Data)",
            Self::Ethiopian(_) => "AnyCalendar (Ethiopian)",
            Self::Indian(_) => "AnyCalendar (Indian)",
            Self::Coptic(_) => "AnyCalendar (Coptic)",
            Self::Iso(_) => "AnyCalendar (Iso)",
        }
    }

    fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
        Some(self.kind())
    }
}

impl AnyCalendar {
    /// Constructs an AnyCalendar for a given calendar kind and [`AnyProvider`] data source
    ///
    /// As this requires a valid [`AnyCalendarKind`] to work, it does not do any kind of locale-based
    /// fallbacking. If this is desired, use [`Self::try_new_for_locale_with_any_provider()`].
    ///
    /// For calendars that need data, will attempt to load the appropriate data from the source.
    ///
    /// This API needs the `calendar/japanese@1` or `calendar/japanext@1` data key if working with Japanese calendars.
    pub fn try_new_with_any_provider<P>(
        provider: &P,
        kind: AnyCalendarKind,
    ) -> Result<Self, CalendarError>
    where
        P: AnyProvider + ?Sized,
    {
        Ok(match kind {
            AnyCalendarKind::Gregorian => AnyCalendar::Gregorian(Gregorian),
            AnyCalendarKind::Buddhist => AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Japanese => {
                AnyCalendar::Japanese(Japanese::try_new_with_any_provider(provider)?)
            }
            AnyCalendarKind::JapaneseExtended => AnyCalendar::JapaneseExtended(
                JapaneseExtended::try_new_with_any_provider(provider)?,
            ),
            AnyCalendarKind::Indian => AnyCalendar::Indian(Indian),
            AnyCalendarKind::Coptic => AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Iso => AnyCalendar::Iso(Iso),
            AnyCalendarKind::Ethiopian => AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                EthiopianEraStyle::AmeteMihret,
            )),
            AnyCalendarKind::EthiopianAmeteAlem => {
                AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(EthiopianEraStyle::AmeteAlem))
            }
        })
    }

    /// Constructs an AnyCalendar for a given calendar kind and [`BufferProvider`] data source
    ///
    /// As this requires a valid [`AnyCalendarKind`] to work, it does not do any kind of locale-based
    /// fallbacking. If this is desired, use [`Self::try_new_for_locale_with_buffer_provider()`].
    ///
    /// For calendars that need data, will attempt to load the appropriate data from the source.
    ///
    /// This API needs the `calendar/japanese@1` or `calendar/japanext@1` data key if working with Japanese calendars.
    ///
    /// This needs the `"serde"` Cargo feature to be enabled to be used
    #[cfg(feature = "serde")]
    pub fn try_new_with_buffer_provider<P>(
        provider: &P,
        kind: AnyCalendarKind,
    ) -> Result<Self, CalendarError>
    where
        P: BufferProvider + ?Sized,
    {
        Ok(match kind {
            AnyCalendarKind::Gregorian => AnyCalendar::Gregorian(Gregorian),
            AnyCalendarKind::Buddhist => AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Japanese => {
                AnyCalendar::Japanese(Japanese::try_new_with_buffer_provider(provider)?)
            }
            AnyCalendarKind::JapaneseExtended => AnyCalendar::JapaneseExtended(
                JapaneseExtended::try_new_with_buffer_provider(provider)?,
            ),
            AnyCalendarKind::Indian => AnyCalendar::Indian(Indian),
            AnyCalendarKind::Coptic => AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Iso => AnyCalendar::Iso(Iso),
            AnyCalendarKind::Ethiopian => AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                EthiopianEraStyle::AmeteMihret,
            )),
            AnyCalendarKind::EthiopianAmeteAlem => {
                AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(EthiopianEraStyle::AmeteAlem))
            }
        })
    }

    /// Constructs an AnyCalendar for a given calendar kind and data source.
    ///
    /// **This method is unstable; the bounds on `P` might expand over time as more calendars are added**
    ///
    /// As this requires a valid [`AnyCalendarKind`] to work, it does not do any kind of locale-based
    /// fallbacking. If this is desired, use [`Self::try_new_for_locale_unstable()`].
    ///
    /// For calendars that need data, will attempt to load the appropriate data from the source
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    /// <div class="stab unstable">
    /// ⚠️ The bounds on this function may change over time, including in SemVer minor releases.
    /// </div>
    pub fn try_new_unstable<P>(provider: &P, kind: AnyCalendarKind) -> Result<Self, CalendarError>
    where
        P: DataProvider<crate::provider::JapaneseErasV1Marker>
            + DataProvider<crate::provider::JapaneseExtendedErasV1Marker>
            + ?Sized,
    {
        Ok(match kind {
            AnyCalendarKind::Gregorian => AnyCalendar::Gregorian(Gregorian),
            AnyCalendarKind::Buddhist => AnyCalendar::Buddhist(Buddhist),
            AnyCalendarKind::Japanese => {
                AnyCalendar::Japanese(Japanese::try_new_unstable(provider)?)
            }
            AnyCalendarKind::JapaneseExtended => {
                AnyCalendar::JapaneseExtended(JapaneseExtended::try_new_unstable(provider)?)
            }
            AnyCalendarKind::Indian => AnyCalendar::Indian(Indian),
            AnyCalendarKind::Coptic => AnyCalendar::Coptic(Coptic),
            AnyCalendarKind::Iso => AnyCalendar::Iso(Iso),
            AnyCalendarKind::Ethiopian => AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(
                EthiopianEraStyle::AmeteMihret,
            )),
            AnyCalendarKind::EthiopianAmeteAlem => {
                AnyCalendar::Ethiopian(Ethiopian::new_with_era_style(EthiopianEraStyle::AmeteAlem))
            }
        })
    }

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

    /// Constructs an AnyCalendar for a given calendar kind and data source.
    ///
    /// **This method is unstable; the bounds on `P` might expand over time as more calendars are added**
    ///
    /// In case the locale's calendar is unknown or unspecified, it will attempt to load the default
    /// calendar for the locale, falling back to gregorian.
    ///
    /// For calendars that need data, will attempt to load the appropriate data from the source
    pub fn try_new_for_locale_unstable<P>(
        provider: &P,
        locale: &DataLocale,
    ) -> Result<Self, CalendarError>
    where
        P: DataProvider<crate::provider::JapaneseErasV1Marker>
            + DataProvider<crate::provider::JapaneseExtendedErasV1Marker>
            + ?Sized,
    {
        let kind = AnyCalendarKind::from_data_locale_with_fallback(locale);
        Self::try_new_unstable(provider, kind)
    }

    fn calendar_name(&self) -> &'static str {
        match *self {
            Self::Gregorian(_) => "Gregorian",
            Self::Buddhist(_) => "Buddhist",
            Self::Japanese(_) => "Japanese",
            Self::JapaneseExtended(_) => "Japanese (Historical era data)",
            Self::Ethiopian(_) => "Ethiopian",
            Self::Indian(_) => "Indian",
            Self::Coptic(_) => "Coptic",
            Self::Iso(_) => "Iso",
        }
    }

    /// The [`AnyCalendarKind`] corresponding to the calendar this contains
    pub fn kind(&self) -> AnyCalendarKind {
        match *self {
            Self::Gregorian(_) => AnyCalendarKind::Gregorian,
            Self::Buddhist(_) => AnyCalendarKind::Buddhist,
            Self::Japanese(_) => AnyCalendarKind::Japanese,
            Self::JapaneseExtended(_) => AnyCalendarKind::JapaneseExtended,
            #[allow(clippy::expect_used)] // Invariant known at compile time
            Self::Ethiopian(ref e) => e
                .any_calendar_kind()
                .expect("Ethiopian calendar known to have an AnyCalendarKind"),
            Self::Indian(_) => AnyCalendarKind::Indian,
            Self::Coptic(_) => AnyCalendarKind::Coptic,
            Self::Iso(_) => AnyCalendarKind::Iso,
        }
    }

    /// Given an AnyCalendar date, convert that date to another AnyCalendar date in this calendar,
    /// if conversion is needed
    pub fn convert_any_date<'a>(
        &'a self,
        date: &Date<impl AsCalendar<Calendar = AnyCalendar>>,
    ) -> Date<Ref<'a, AnyCalendar>> {
        if self.kind() != date.calendar.as_calendar().kind() {
            Date::new_from_iso(date.to_iso(), Ref(self))
        } else {
            Date {
                inner: date.inner.clone(),
                calendar: Ref(self),
            }
        }
    }

    /// Given an AnyCalendar datetime, convert that date to another AnyCalendar datetime in this calendar,
    /// if conversion is needed
    pub fn convert_any_datetime<'a>(
        &'a self,
        date: &DateTime<impl AsCalendar<Calendar = AnyCalendar>>,
    ) -> DateTime<Ref<'a, AnyCalendar>> {
        DateTime {
            time: date.time,
            date: self.convert_any_date(&date.date),
        }
    }
}

impl AnyDateInner {
    fn calendar_name(&self) -> &'static str {
        match *self {
            AnyDateInner::Gregorian(_) => "Gregorian",
            AnyDateInner::Buddhist(_) => "Buddhist",
            AnyDateInner::Japanese(_) => "Japanese",
            AnyDateInner::JapaneseExtended(_) => "Japanese (Historical era data)",
            AnyDateInner::Ethiopian(_) => "Ethiopian",
            AnyDateInner::Indian(_) => "Indian",
            AnyDateInner::Coptic(_) => "Coptic",
            AnyDateInner::Iso(_) => "Iso",
        }
    }
}

/// Convenient type for selecting the kind of AnyCalendar to construct
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AnyCalendarKind {
    /// The kind of a [`Gregorian`] calendar
    Gregorian,
    /// The kind of a [`Buddhist`] calendar
    Buddhist,
    /// The kind of a [`Japanese`] calendar
    Japanese,
    /// The kind of a [`JapaneseExtended`] calendar
    JapaneseExtended,
    /// The kind of an [`Ethiopian`] calendar, with Amete Mihret era
    Ethiopian,
    /// The kind of an [`Ethiopian`] calendar, with Amete Alem era
    EthiopianAmeteAlem,
    /// The kind of a [`Indian`] calendar
    Indian,
    /// The kind of a [`Coptic`] calendar
    Coptic,
    /// The kind of an [`Iso`] calendar
    Iso,
}

impl AnyCalendarKind {
    /// Construct from a BCP-47 string
    ///
    /// Returns None if the calendar is unknown. If you prefer an error, use
    /// [`CalendarError::unknown_any_calendar_kind`].
    pub fn get_for_bcp47_string(x: &str) -> Option<Self> {
        Self::get_for_bcp47_bytes(x.as_bytes())
    }
    /// Construct from a BCP-47 byte string
    ///
    /// Returns None if the calendar is unknown. If you prefer an error, use
    /// [`CalendarError::unknown_any_calendar_kind`].
    pub fn get_for_bcp47_bytes(x: &[u8]) -> Option<Self> {
        Some(match x {
            b"gregory" => AnyCalendarKind::Gregorian,
            b"buddhist" => AnyCalendarKind::Buddhist,
            b"japanese" => AnyCalendarKind::Japanese,
            b"japanext" => AnyCalendarKind::JapaneseExtended,
            b"indian" => AnyCalendarKind::Indian,
            b"coptic" => AnyCalendarKind::Coptic,
            b"iso" => AnyCalendarKind::Iso,
            b"ethiopic" => AnyCalendarKind::Ethiopian,
            b"ethioaa" => AnyCalendarKind::EthiopianAmeteAlem,
            _ => return None,
        })
    }
    /// Construct from a BCP-47 [`Value`]
    ///
    /// Returns None if the calendar is unknown. If you prefer an error, use
    /// [`CalendarError::unknown_any_calendar_kind`].
    pub fn get_for_bcp47_value(x: &Value) -> Option<Self> {
        Some(if *x == value!("gregory") {
            AnyCalendarKind::Gregorian
        } else if *x == value!("buddhist") {
            AnyCalendarKind::Buddhist
        } else if *x == value!("japanese") {
            AnyCalendarKind::Japanese
        } else if *x == value!("japanext") {
            AnyCalendarKind::JapaneseExtended
        } else if *x == value!("indian") {
            AnyCalendarKind::Indian
        } else if *x == value!("coptic") {
            AnyCalendarKind::Coptic
        } else if *x == value!("iso") {
            AnyCalendarKind::Iso
        } else if *x == value!("ethiopic") {
            AnyCalendarKind::Ethiopian
        } else if *x == value!("ethioaa") {
            AnyCalendarKind::EthiopianAmeteAlem
        } else {
            return None;
        })
    }

    /// Convert to a BCP-47 string
    pub fn as_bcp47_string(self) -> &'static str {
        match self {
            AnyCalendarKind::Gregorian => "gregory",
            AnyCalendarKind::Buddhist => "buddhist",
            AnyCalendarKind::Japanese => "japanese",
            AnyCalendarKind::JapaneseExtended => "japanext",
            AnyCalendarKind::Indian => "indian",
            AnyCalendarKind::Coptic => "coptic",
            AnyCalendarKind::Iso => "iso",
            AnyCalendarKind::Ethiopian => "ethiopic",
            AnyCalendarKind::EthiopianAmeteAlem => "ethioaa",
        }
    }

    /// Convert to a BCP-47 `Value`
    pub fn as_bcp47_value(self) -> Value {
        match self {
            AnyCalendarKind::Gregorian => value!("gregory"),
            AnyCalendarKind::Buddhist => value!("buddhist"),
            AnyCalendarKind::Japanese => value!("japanese"),
            AnyCalendarKind::JapaneseExtended => value!("japanext"),
            AnyCalendarKind::Indian => value!("indian"),
            AnyCalendarKind::Coptic => value!("coptic"),
            AnyCalendarKind::Iso => value!("iso"),
            AnyCalendarKind::Ethiopian => value!("ethiopic"),
            AnyCalendarKind::EthiopianAmeteAlem => value!("ethioaa"),
        }
    }

    /// Extract the calendar component from a [`Locale`]
    ///
    /// Returns None if the calendar is not specified or unknown. If you prefer an error, use
    /// [`CalendarError::unknown_any_calendar_kind`].
    pub fn get_for_locale(l: &Locale) -> Option<Self> {
        l.extensions
            .unicode
            .keywords
            .get(&key!("ca"))
            .and_then(Self::get_for_bcp47_value)
    }

    /// Extract the calendar component from a [`DataLocale`]
    ///
    /// Returns None if the calendar is not specified or unknown. If you prefer an error, use
    /// [`CalendarError::unknown_any_calendar_kind`].
    fn get_for_data_locale(l: &DataLocale) -> Option<Self> {
        l.get_unicode_ext(&key!("ca"))
            .and_then(|v| Self::get_for_bcp47_value(&v))
    }

    // Do not make public, this will eventually need fallback
    // data from the provider
    fn from_data_locale_with_fallback(l: &DataLocale) -> Self {
        if let Some(kind) = Self::get_for_data_locale(l) {
            kind
        } else {
            let lang = l.language();
            if lang == language!("th") {
                Self::Buddhist
            // Other known fallback routes for currently-unsupported calendars
            // } else if lang == language!("sa") {
            //     Self::IslamicUmalqura
            // } else if lang == language!("af") || lang == language!("ir") {
            //     Self::Persian
            } else {
                Self::Gregorian
            }
        }
    }
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

/// Trait for calendars that may be converted to [`AnyCalendar`]
pub trait IntoAnyCalendar: Calendar + Sized {
    /// Convert this calendar into an [`AnyCalendar`], moving it
    ///
    /// You should not need to call this method directly
    fn to_any(self) -> AnyCalendar;

    /// Convert this calendar into an [`AnyCalendar`], cloning it
    ///
    /// You should not need to call this method directly
    fn to_any_cloned(&self) -> AnyCalendar;
    /// Convert a date for this calendar into an [`AnyDateInner`]
    ///
    /// You should not need to call this method directly
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner;
}

impl IntoAnyCalendar for Gregorian {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Gregorian(Gregorian)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Gregorian(Gregorian)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Gregorian(*d)
    }
}

impl IntoAnyCalendar for Buddhist {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Buddhist(Buddhist)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Buddhist(Buddhist)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Buddhist(*d)
    }
}

impl IntoAnyCalendar for Japanese {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Japanese(self)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Japanese(self.clone())
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Japanese(*d)
    }
}

impl IntoAnyCalendar for JapaneseExtended {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::JapaneseExtended(self)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::JapaneseExtended(self.clone())
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::JapaneseExtended(*d)
    }
}

impl IntoAnyCalendar for Ethiopian {
    // Amete Mihret calendars are the default
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Ethiopian(self)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Ethiopian(*self)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Ethiopian(*d)
    }
}

impl IntoAnyCalendar for Indian {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Indian(Indian)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Indian(Indian)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Indian(*d)
    }
}

impl IntoAnyCalendar for Coptic {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Coptic(Coptic)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Coptic(Coptic)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Coptic(*d)
    }
}

impl IntoAnyCalendar for Iso {
    fn to_any(self) -> AnyCalendar {
        AnyCalendar::Iso(Iso)
    }
    fn to_any_cloned(&self) -> AnyCalendar {
        AnyCalendar::Iso(Iso)
    }
    fn date_to_any(&self, d: &Self::DateInner) -> AnyDateInner {
        AnyDateInner::Iso(*d)
    }
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
    ) {
        let era = types::Era(era.parse().expect("era must parse"));
        let month = types::MonthCode(month_code.parse().expect("month code must parse"));

        let date = Date::try_new_from_codes(era, year, month, day, calendar).unwrap_or_else(|e| {
            panic!(
                "Failed to construct date for {} with {:?}, {}, {}, {}: {}",
                calendar.debug_name(),
                era,
                year,
                month,
                day,
                e,
            )
        });

        let roundtrip_year = date.year();
        let roundtrip_era = roundtrip_year.era;
        let roundtrip_year = roundtrip_year.number;
        let roundtrip_month = date.month().code;
        let roundtrip_day = date.day_of_month().0.try_into().expect("Must fit in u8");

        assert_eq!(
            (era, year, month, day),
            (
                roundtrip_era,
                roundtrip_year,
                roundtrip_month,
                roundtrip_day
            ),
            "Failed to roundtrip for calendar {}",
            calendar.debug_name()
        );

        let iso = date.to_iso();
        let reconstructed = Date::new_from_iso(iso, calendar);
        assert_eq!(
            date, reconstructed,
            "Failed to roundtrip via iso with {era:?}, {year}, {month}, {day}"
        )
    }

    fn single_test_error(
        calendar: Ref<AnyCalendar>,
        era: &str,
        year: i32,
        month_code: &str,
        day: u8,
        error: CalendarError,
    ) {
        let era = types::Era(era.parse().expect("era must parse"));
        let month = types::MonthCode(month_code.parse().expect("month code must parse"));

        let date = Date::try_new_from_codes(era, year, month, day, calendar);
        assert_eq!(
            date,
            Err(error),
            "Construction with {era:?}, {year}, {month}, {day} did not return {error:?}"
        )
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_any_construction() {
        let buddhist = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Buddhist,
        )
        .expect("Calendar construction must succeed");
        let coptic = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Coptic,
        )
        .expect("Calendar construction must succeed");
        let ethiopian = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Ethiopian,
        )
        .expect("Calendar construction must succeed");
        let ethioaa = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::EthiopianAmeteAlem,
        )
        .expect("Calendar construction must succeed");
        let gregorian = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Gregorian,
        )
        .expect("Calendar construction must succeed");
        let indian = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Indian,
        )
        .expect("Calendar construction must succeed");
        let japanese = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::Japanese,
        )
        .expect("Calendar construction must succeed");
        let japanext = AnyCalendar::try_new_with_buffer_provider(
            &icu_testdata::buffer(),
            AnyCalendarKind::JapaneseExtended,
        )
        .expect("Calendar construction must succeed");
        let buddhist = Ref(&buddhist);
        let coptic = Ref(&coptic);
        let ethiopian = Ref(&ethiopian);
        let ethioaa = Ref(&ethioaa);
        let gregorian = Ref(&gregorian);
        let indian = Ref(&indian);
        let japanese = Ref(&japanese);
        let japanext = Ref(&japanext);

        single_test_roundtrip(buddhist, "be", 100, "M03", 1);
        single_test_roundtrip(buddhist, "be", 2000, "M03", 1);
        single_test_roundtrip(buddhist, "be", -100, "M03", 1);
        single_test_error(
            buddhist,
            "be",
            100,
            "M13",
            1,
            CalendarError::UnknownMonthCode("M13".parse().unwrap(), "Buddhist"),
        );

        single_test_roundtrip(coptic, "ad", 100, "M03", 1);
        single_test_roundtrip(coptic, "ad", 2000, "M03", 1);
        // fails ISO roundtrip
        // single_test_roundtrip(coptic, "bd", 100, "M03", 1);
        single_test_roundtrip(coptic, "ad", 100, "M13", 1);
        single_test_error(
            coptic,
            "ad",
            100,
            "M14",
            1,
            CalendarError::UnknownMonthCode("M14".parse().unwrap(), "Coptic"),
        );
        single_test_error(coptic, "ad", 0, "M03", 1, CalendarError::OutOfRange);
        single_test_error(coptic, "bd", 0, "M03", 1, CalendarError::OutOfRange);

        single_test_roundtrip(ethiopian, "incar", 100, "M03", 1);
        single_test_roundtrip(ethiopian, "incar", 2000, "M03", 1);
        single_test_roundtrip(ethiopian, "incar", 2000, "M13", 1);
        // Fails ISO roundtrip due to https://github.com/unicode-org/icu4x/issues/2254
        // single_test_roundtrip(ethiopian, "pre-incar", 100, "M03", 1);
        single_test_error(ethiopian, "incar", 0, "M03", 1, CalendarError::OutOfRange);
        single_test_error(
            ethiopian,
            "pre-incar",
            0,
            "M03",
            1,
            CalendarError::OutOfRange,
        );
        single_test_error(
            ethiopian,
            "incar",
            100,
            "M14",
            1,
            CalendarError::UnknownMonthCode("M14".parse().unwrap(), "Ethiopian"),
        );

        single_test_roundtrip(ethioaa, "mundi", 7000, "M13", 1);
        single_test_roundtrip(ethioaa, "mundi", 7000, "M13", 1);
        // Fails ISO roundtrip due to https://github.com/unicode-org/icu4x/issues/2254
        // single_test_roundtrip(ethioaa, "mundi", 100, "M03", 1);
        single_test_error(
            ethiopian,
            "mundi",
            100,
            "M14",
            1,
            CalendarError::UnknownMonthCode("M14".parse().unwrap(), "Ethiopian"),
        );

        single_test_roundtrip(gregorian, "ce", 100, "M03", 1);
        single_test_roundtrip(gregorian, "ce", 2000, "M03", 1);
        single_test_roundtrip(gregorian, "bce", 100, "M03", 1);
        single_test_error(gregorian, "ce", 0, "M03", 1, CalendarError::OutOfRange);
        single_test_error(gregorian, "bce", 0, "M03", 1, CalendarError::OutOfRange);

        single_test_error(
            gregorian,
            "bce",
            100,
            "M13",
            1,
            CalendarError::UnknownMonthCode("M13".parse().unwrap(), "Gregorian"),
        );

        single_test_roundtrip(indian, "saka", 100, "M03", 1);
        single_test_roundtrip(indian, "saka", 2000, "M12", 1);
        single_test_roundtrip(indian, "saka", -100, "M03", 1);
        single_test_roundtrip(indian, "saka", 0, "M03", 1);
        single_test_error(
            indian,
            "saka",
            100,
            "M13",
            1,
            CalendarError::UnknownMonthCode("M13".parse().unwrap(), "Indian"),
        );
        single_test_roundtrip(japanese, "reiwa", 3, "M03", 1);
        single_test_roundtrip(japanese, "heisei", 6, "M12", 1);
        single_test_roundtrip(japanese, "meiji", 10, "M03", 1);
        single_test_roundtrip(japanese, "ce", 1000, "M03", 1);
        single_test_roundtrip(japanese, "bce", 10, "M03", 1);
        single_test_error(japanese, "ce", 0, "M03", 1, CalendarError::OutOfRange);
        single_test_error(japanese, "bce", 0, "M03", 1, CalendarError::OutOfRange);

        single_test_error(
            japanese,
            "reiwa",
            2,
            "M13",
            1,
            CalendarError::UnknownMonthCode("M13".parse().unwrap(), "Japanese (Modern eras only)"),
        );

        single_test_roundtrip(japanext, "reiwa", 3, "M03", 1);
        single_test_roundtrip(japanext, "heisei", 6, "M12", 1);
        single_test_roundtrip(japanext, "meiji", 10, "M03", 1);
        single_test_roundtrip(japanext, "tenpyokampo-749", 1, "M04", 20);
        single_test_roundtrip(japanext, "ce", 100, "M03", 1);
        single_test_roundtrip(japanext, "bce", 10, "M03", 1);
        single_test_error(japanext, "ce", 0, "M03", 1, CalendarError::OutOfRange);
        single_test_error(japanext, "bce", 0, "M03", 1, CalendarError::OutOfRange);

        single_test_error(
            japanext,
            "reiwa",
            2,
            "M13",
            1,
            CalendarError::UnknownMonthCode(
                "M13".parse().unwrap(),
                "Japanese (With historical eras)",
            ),
        );
    }
}

}
pub mod buddhist {
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and implementations for the Buddhist calendar.
//!
//! ```rust
//! use icu::calendar::{buddhist::Buddhist, Date, DateTime};
//!
//! // `Date` type
//! let date_iso = Date::try_new_iso_date(1970, 1, 2)
//!     .expect("Failed to initialize ISO Date instance.");
//! let date_buddhist = Date::new_from_iso(date_iso, Buddhist);
//!
//! // `DateTime` type
//! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
//!     .expect("Failed to initialize ISO DateTime instance.");
//! let datetime_buddhist = DateTime::new_from_iso(datetime_iso, Buddhist);
//!
//! // `Date` checks
//! assert_eq!(date_buddhist.year().number, 2513);
//! assert_eq!(date_buddhist.month().ordinal, 1);
//! assert_eq!(date_buddhist.day_of_month().0, 2);
//!
//! // `DateTime` type
//! assert_eq!(datetime_buddhist.date.year().number, 2513);
//! assert_eq!(datetime_buddhist.date.month().ordinal, 1);
//! assert_eq!(datetime_buddhist.date.day_of_month().0, 2);
//! assert_eq!(datetime_buddhist.time.hour.number(), 13);
//! assert_eq!(datetime_buddhist.time.minute.number(), 1);
//! assert_eq!(datetime_buddhist.time.second.number(), 0);
//! ```

use crate::any_calendar::AnyCalendarKind;
use crate::calendar_arithmetic::ArithmeticDate;
use crate::iso::{Iso, IsoDateInner};
use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
use tinystr::tinystr;

/// The number of years the Buddhist Era is ahead of C.E. by
///
/// (1 AD = 544 BE)
const BUDDHIST_ERA_OFFSET: i32 = 543;

#[derive(Copy, Clone, Debug, Default)]
/// The [Thai Solar Buddhist Calendar][cal]
///
/// The [Thai Solar Buddhist Calendar][cal] is a solar calendar used in Thailand, with twelve months.
/// The months and days are identical to that of the Gregorian calendar, however the years are counted
/// differently using the Buddhist Era.
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// [cal]: https://en.wikipedia.org/wiki/Thai_solar_calendar
///
/// # Era codes
///
/// This calendar supports one era, `"be"`, with 1 B.E. being 543 BCE

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
    ) -> Result<Self::DateInner, CalendarError> {
        if era.0 != tinystr!(16, "be") {
            return Err(CalendarError::UnknownEra(era.0, self.debug_name()));
        }
        let year = year - BUDDHIST_ERA_OFFSET;

        ArithmeticDate::new_from_solar(self, year, month_code, day).map(IsoDateInner)
    }
    fn date_from_iso(&self, iso: Date<Iso>) -> IsoDateInner {
        *iso.inner()
    }

    fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso> {
        Date::from_raw(*date, Iso)
    }

    fn months_in_year(&self, date: &Self::DateInner) -> u8 {
        Iso.months_in_year(date)
    }

    fn days_in_year(&self, date: &Self::DateInner) -> u32 {
        Iso.days_in_year(date)
    }

    fn days_in_month(&self, date: &Self::DateInner) -> u8 {
        Iso.days_in_month(date)
    }

    fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>) {
        Iso.offset_date(date, offset.cast_unit())
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
        Iso.until(date1, date2, &Iso, largest_unit, smallest_unit)
            .cast_unit()
    }

    /// The calendar-specific year represented by `date`
    fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
        iso_year_as_buddhist(date.0.year)
    }

    /// The calendar-specific month represented by `date`
    fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
        Iso.month(date)
    }

    /// The calendar-specific day-of-month represented by `date`
    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        Iso.day_of_month(date)
    }

    /// Information of the day of the year
    fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo {
        let prev_year = date.0.year - 1;
        let next_year = date.0.year + 1;
        types::DayOfYearInfo {
            day_of_year: Iso::day_of_year(*date),
            days_in_year: Iso::days_in_year_direct(date.0.year),
            prev_year: iso_year_as_buddhist(prev_year),
            days_in_prev_year: Iso::days_in_year_direct(prev_year),
            next_year: iso_year_as_buddhist(next_year),
        }
    }

    fn debug_name(&self) -> &'static str {
        "Buddhist"
    }

    fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
        Some(AnyCalendarKind::Buddhist)
    }
}

impl Date<Buddhist> {
    /// Construct a new Buddhist Date.
    ///
    /// Years are specified as BE years.
    ///
    /// ```rust
    /// use icu::calendar::Date;
    /// use std::convert::TryFrom;
    ///
    /// let date_buddhist = Date::try_new_buddhist_date(1970, 1, 2)
    ///     .expect("Failed to initialize Buddhist Date instance.");
    ///
    /// assert_eq!(date_buddhist.year().number, 1970);
    /// assert_eq!(date_buddhist.month().ordinal, 1);
    /// assert_eq!(date_buddhist.day_of_month().0, 2);
    /// ```
    pub fn try_new_buddhist_date(
        year: i32,
        month: u8,
        day: u8,
    ) -> Result<Date<Buddhist>, CalendarError> {
        Date::try_new_iso_date(year - BUDDHIST_ERA_OFFSET, month, day)
            .map(|d| Date::new_from_iso(d, Buddhist))
    }
}

impl DateTime<Buddhist> {
    /// Construct a new Buddhist datetime from integers.
    ///
    /// Years are specified as BE years.
    ///
    /// ```rust
    /// use icu::calendar::DateTime;
    ///
    /// let datetime_buddhist =
    ///     DateTime::try_new_buddhist_datetime(1970, 1, 2, 13, 1, 0)
    ///         .expect("Failed to initialize Buddhist DateTime instance.");
    ///
    /// assert_eq!(datetime_buddhist.date.year().number, 1970);
    /// assert_eq!(datetime_buddhist.date.month().ordinal, 1);
    /// assert_eq!(datetime_buddhist.date.day_of_month().0, 2);
    /// assert_eq!(datetime_buddhist.time.hour.number(), 13);
    /// assert_eq!(datetime_buddhist.time.minute.number(), 1);
    /// assert_eq!(datetime_buddhist.time.second.number(), 0);
    /// ```
    pub fn try_new_buddhist_datetime(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<DateTime<Buddhist>, CalendarError> {
        Ok(DateTime {
            date: Date::try_new_buddhist_date(year, month, day)?,
            time: types::Time::try_new(hour, minute, second, 0)?,
        })
    }
}

fn iso_year_as_buddhist(year: i32) -> types::FormattableYear {
    let buddhist_year = year + BUDDHIST_ERA_OFFSET;
    types::FormattableYear {
        era: types::Era(tinystr!(16, "be")),
        number: buddhist_year,
        related_iso: None,
    }
}

}
mod calendar {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    use crate::any_calendar::AnyCalendarKind;
    use crate::{types, CalendarError, Date, DateDuration, DateDurationUnit, Iso};
    use core::fmt;
    
    /// A calendar implementation
    ///
    /// Only implementors of [`Calendar`] should care about these methods, in general users of
    /// these calendars should use the methods on [`Date`] instead.
    ///
    /// Individual [`Calendar`] implementations may have inherent utility methods
    /// allowing for direct construction, etc.
    ///
    /// For ICU4X 1.0, implementing this trait or calling methods directly is considered
    /// unstable and prone to change, especially for `offset_date()` and `until()`.
    pub trait Calendar {
        /// The internal type used to represent dates
        type DateInner: PartialEq + Eq + Clone + fmt::Debug;
        /// Construct a date from era/month codes and fields
        fn date_from_codes(
            &self,
            era: types::Era,
            year: i32,
            month_code: types::MonthCode,
            day: u8,
        ) -> Result<Self::DateInner, CalendarError>;
        /// Construct the date from an ISO date
        fn date_from_iso(&self, iso: Date<Iso>) -> Self::DateInner;
        /// Obtain an ISO date from this date
        fn date_to_iso(&self, date: &Self::DateInner) -> Date<Iso>;
        // fn validate_date(&self, e: Era, y: Year, m: MonthCode, d: Day) -> bool;
        // // similar validators for YearMonth, etc
    
        // fn is_leap<A: AsCalendar<Calendar = Self>>(&self, date: &Date<A>) -> bool;
        /// Count the number of months in a given year, specified by providing a date
        /// from that year
        fn months_in_year(&self, date: &Self::DateInner) -> u8;
        /// Count the number of days in a given year, specified by providing a date
        /// from that year
        fn days_in_year(&self, date: &Self::DateInner) -> u32;
        /// Count the number of days in a given month, specified by providing a date
        /// from that year/month
        fn days_in_month(&self, date: &Self::DateInner) -> u8;
        /// Calculate the day of the week and return it
        fn day_of_week(&self, date: &Self::DateInner) -> types::IsoWeekday {
            self.date_to_iso(date).day_of_week()
        }
        // fn week_of_year(&self, date: &Self::DateInner) -> u8;
    
        #[doc(hidden)] // unstable
        /// Add `offset` to `date`
        fn offset_date(&self, date: &mut Self::DateInner, offset: DateDuration<Self>);
    
        #[doc(hidden)] // unstable
        /// Calculate `date2 - date` as a duration
        ///
        /// `calendar2` is the calendar object associated with `date2`. In case the specific calendar objects
        /// differ on data, the data for the first calendar is used, and `date2` may be converted if necessary.
        fn until(
            &self,
            date1: &Self::DateInner,
            date2: &Self::DateInner,
            calendar2: &Self,
            largest_unit: DateDurationUnit,
            smallest_unit: DateDurationUnit,
        ) -> DateDuration<Self>;
    
        /// Obtain a name for the calendar for debug printing
        fn debug_name(&self) -> &'static str;
        // fn since(&self, from: &Date<Self>, to: &Date<Self>) -> Duration<Self>, Error;
    
        /// The calendar-specific year represented by `date`
        fn year(&self, date: &Self::DateInner) -> types::FormattableYear;
    
        /// The calendar-specific month represented by `date`
        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth;
    
        /// The calendar-specific day-of-month represented by `date`
        fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth;
    
        /// Information of the day of the year
        fn day_of_year_info(&self, date: &Self::DateInner) -> types::DayOfYearInfo;
    
        /// The [`AnyCalendarKind`] corresponding to this calendar,
        /// if one exists. Implementors outside of icu_calendar should return None
        fn any_calendar_kind(&self) -> Option<AnyCalendarKind> {
            None
        }
    }
    }
mod calendar_arithmetic {
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::{types, Calendar, CalendarError, DateDuration, DateDurationUnit};
use core::convert::TryInto;
use core::marker::PhantomData;
use tinystr::tinystr;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[allow(clippy::exhaustive_structs)] // this type is stable
pub struct ArithmeticDate<C: CalendarArithmetic> {
    pub year: i32,
    /// 1-based month of year
    pub month: u8,
    /// 1-based day of month
    pub day: u8,
    pub marker: PhantomData<C>,
}

pub trait CalendarArithmetic: Calendar {
    fn month_days(year: i32, month: u8) -> u8;
    fn months_for_every_year(year: i32) -> u8;
    fn is_leap_year(year: i32) -> bool;

    /// Calculate the days in a given year
    /// Can be overridden with simpler implementations for solar calendars
    /// (typically, 366 in leap, 365 otgerwuse) Leave this as the default
    /// for lunar calendars
    ///
    /// The name has `provided` in it to avoid clashes with Calendar
    fn days_in_provided_year(year: i32) -> u32 {
        let months_in_year = Self::months_for_every_year(year);
        let mut days: u32 = 0;
        for month in 1..=months_in_year {
            days += Self::month_days(year, month) as u32;
        }
        days
    }
}

impl<C: CalendarArithmetic> ArithmeticDate<C> {
    #[inline]
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        ArithmeticDate {
            year,
            month,
            day,
            marker: PhantomData,
        }
    }

    #[inline]
    fn offset_days(&mut self, mut day_offset: i32) {
        while day_offset != 0 {
            let month_days = C::month_days(self.year, self.month);
            if self.day as i32 + day_offset > month_days as i32 {
                self.offset_months(1);
                day_offset -= month_days as i32;
            } else if self.day as i32 + day_offset < 1 {
                self.offset_months(-1);
                day_offset += C::month_days(self.year, self.month) as i32;
            } else {
                self.day = (self.day as i32 + day_offset) as u8;
                day_offset = 0;
            }
        }
    }

    #[inline]
    fn offset_months(&mut self, mut month_offset: i32) {
        while month_offset != 0 {
            let year_months = C::months_for_every_year(self.year);
            if self.month as i32 + month_offset > year_months as i32 {
                self.year += 1;
                month_offset -= year_months as i32;
            } else if self.month as i32 + month_offset < 1 {
                self.year -= 1;
                month_offset += C::months_for_every_year(self.year) as i32;
            } else {
                self.month = (self.month as i32 + month_offset) as u8;
                month_offset = 0
            }
        }
    }

    #[inline]
    pub fn offset_date(&mut self, offset: DateDuration<C>) {
        // For offset_date to work with lunar calendars, need to handle an edge case where the original month is not valid in the future year.
        self.year += offset.years;

        self.offset_months(offset.months);

        let day_offset = offset.days + offset.weeks * 7 + self.day as i32 - 1;
        self.day = 1;
        self.offset_days(day_offset);
    }

    #[inline]
    pub fn until(
        &self,
        date2: ArithmeticDate<C>,
        _largest_unit: DateDurationUnit,
        _smaller_unti: DateDurationUnit,
    ) -> DateDuration<C> {
        DateDuration::new(
            self.year - date2.year,
            self.month as i32 - date2.month as i32,
            0,
            self.day as i32 - date2.day as i32,
        )
    }

    #[inline]
    pub fn days_in_year(&self) -> u32 {
        C::days_in_provided_year(self.year)
    }

    #[inline]
    pub fn months_in_year(&self) -> u8 {
        C::months_for_every_year(self.year)
    }

    #[inline]
    pub fn days_in_month(&self) -> u8 {
        C::month_days(self.year, self.month)
    }

    #[inline]
    pub fn day_of_year(&self) -> u32 {
        let mut day_of_year = 0;
        for month in 1..self.month {
            day_of_year += C::month_days(self.year, month) as u32;
        }
        day_of_year + (self.day as u32)
    }

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
        // The day is expected to be within the range of month_days of C
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

    /// The [`types::FormattableMonth`] for the current month (with month code) for a solar calendar
    /// Lunar calendars should not use this method and instead manually implement a month code
    /// resolver.
    ///
    /// Returns "und" if run with months that are out of bounds for the current
    /// calendar.
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

    /// Construct a new arithmetic date from a year, month code, and day, bounds checking
    /// the month
    pub fn new_from_solar<C2: Calendar>(
        // Separate type since the debug_name() impl may differ when DateInner types
        // are nested (e.g. in GregorianDateInner)
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

/// For solar calendars, get the month number from the month code
pub fn ordinal_solar_month_from_code(code: types::MonthCode) -> Option<u8> {
    // Match statements on tinystrs are annoying so instead
    // we calculate it from the bytes directly
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
pub mod coptic {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    //! This module contains types and implementations for the Coptic calendar.
    //!
    //! ```rust
    //! use icu::calendar::{coptic::Coptic, Date, DateTime};
    //!
    //! // `Date` type
    //! let date_iso = Date::try_new_iso_date(1970, 1, 2)
    //!     .expect("Failed to initialize ISO Date instance.");
    //! let date_coptic = Date::new_from_iso(date_iso, Coptic);
    //!
    //! // `DateTime` type
    //! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
    //!     .expect("Failed to initialize ISO DateTime instance.");
    //! let datetime_coptic = DateTime::new_from_iso(datetime_iso, Coptic);
    //!
    //! // `Date` checks
    //! assert_eq!(date_coptic.year().number, 1686);
    //! assert_eq!(date_coptic.month().ordinal, 4);
    //! assert_eq!(date_coptic.day_of_month().0, 24);
    //!
    //! // `DateTime` type
    //! assert_eq!(datetime_coptic.date.year().number, 1686);
    //! assert_eq!(datetime_coptic.date.month().ordinal, 4);
    //! assert_eq!(datetime_coptic.date.day_of_month().0, 24);
    //! assert_eq!(datetime_coptic.time.hour.number(), 13);
    //! assert_eq!(datetime_coptic.time.minute.number(), 1);
    //! assert_eq!(datetime_coptic.time.second.number(), 0);
    //! ```
    
    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::quotient;
    use crate::iso::Iso;
    use crate::julian::Julian;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;
    
    /// The [Coptic Calendar]
    ///
    /// The [Coptic calendar] is a solar calendar used by the Coptic Orthodox Church, with twelve normal months
    /// and a thirteenth small epagomenal month.
    ///
    /// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
    ///
    /// [Coptic calendar]: https://en.wikipedia.org/wiki/Coptic_calendar
    ///
    /// # Era codes
    ///
    /// This calendar supports two era codes: `"bd"`, and `"ad"`, corresponding to the Before Diocletian and After Diocletian/Anno Martyrum
    /// eras. 1 A.M. is equivalent to 284 C.E.
    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Coptic;
    
    /// The inner date type used for representing [`Date`]s of [`Coptic`]. See [`Date`] and [`Coptic`] for more details.
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
        // "Fixed" is a day count representation of calendars staring from Jan 1st of year 1 of the Georgian Calendar.
        // The fixed date algorithms are from
        // Dershowitz, Nachum, and Edward M. Reingold. _Calendrical calculations_. Cambridge University Press, 2008.
        //
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1978
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
    
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1990
        pub(crate) fn coptic_from_fixed(date: i32) -> CopticDateInner {
            let year = quotient(4 * (date - COPTIC_EPOCH) + 1463, 1461);
            let month = (quotient(date - Self::fixed_from_coptic_integers(year, 1, 1), 30) + 1) as u8; // <= 12 < u8::MAX
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
        /// Construct new Coptic Date.
        ///
        /// Negative years are in the B.D. era, starting with 0 = 1 B.D.
        ///
        /// ```rust
        /// use icu::calendar::Date;
        ///
        /// let date_coptic = Date::try_new_coptic_date(1686, 5, 6)
        ///     .expect("Failed to initialize Coptic Date instance.");
        ///
        /// assert_eq!(date_coptic.year().number, 1686);
        /// assert_eq!(date_coptic.month().ordinal, 5);
        /// assert_eq!(date_coptic.day_of_month().0, 6);
        /// ```
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
        /// Construct a new Coptic datetime from integers.
        ///
        /// Negative years are in the B.D. era, starting with 0 = 1 B.D.
        ///
        /// ```rust
        /// use icu::calendar::DateTime;
        ///
        /// let datetime_coptic =
        ///     DateTime::try_new_coptic_datetime(1686, 5, 6, 13, 1, 0)
        ///         .expect("Failed to initialize Coptic DateTime instance.");
        ///
        /// assert_eq!(datetime_coptic.date.year().number, 1686);
        /// assert_eq!(datetime_coptic.date.month().ordinal, 5);
        /// assert_eq!(datetime_coptic.date.day_of_month().0, 6);
        /// assert_eq!(datetime_coptic.time.hour.number(), 13);
        /// assert_eq!(datetime_coptic.time.minute.number(), 1);
        /// assert_eq!(datetime_coptic.time.second.number(), 0);
        /// ```
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
            // https://github.com/unicode-org/icu4x/issues/2254
            let iso_date = Date::try_new_iso_date(-100, 3, 3).unwrap();
            let coptic = iso_date.to_calendar(Coptic);
            let recovered_iso = coptic.to_iso();
            assert_eq!(iso_date, recovered_iso);
        }
    }
    }
mod duration {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    use crate::Calendar;
    use core::fmt;
    use core::marker::PhantomData;
    
    /// A duration between two dates
    ///
    /// Can be used to perform date arithmetic
    ///
    /// # Example
    ///
    /// ```rust
    /// use icu_calendar::{
    ///     types::IsoWeekday, Date, DateDuration, DateDurationUnit,
    /// };
    ///
    /// // Creating ISO date: 1992-09-02.
    /// let mut date_iso = Date::try_new_iso_date(1992, 9, 2)
    ///     .expect("Failed to initialize ISO Date instance.");
    ///
    /// assert_eq!(date_iso.day_of_week(), IsoWeekday::Wednesday);
    /// assert_eq!(date_iso.year().number, 1992);
    /// assert_eq!(date_iso.month().ordinal, 9);
    /// assert_eq!(date_iso.day_of_month().0, 2);
    ///
    /// // Answering questions about days in month and year.
    /// assert_eq!(date_iso.days_in_year(), 366);
    /// assert_eq!(date_iso.days_in_month(), 30);
    ///
    /// // Advancing date in-place by 1 year, 2 months, 3 weeks, 4 days.
    /// date_iso.add(DateDuration::new(1, 2, 3, 4));
    /// assert_eq!(date_iso.year().number, 1993);
    /// assert_eq!(date_iso.month().ordinal, 11);
    /// assert_eq!(date_iso.day_of_month().0, 27);
    ///
    /// // Reverse date advancement.
    /// date_iso.add(DateDuration::new(-1, -2, -3, -4));
    /// assert_eq!(date_iso.year().number, 1992);
    /// assert_eq!(date_iso.month().ordinal, 9);
    /// assert_eq!(date_iso.day_of_month().0, 2);
    ///
    /// // Creating ISO date: 2022-01-30.
    /// let newer_date_iso = Date::try_new_iso_date(2022, 1, 30)
    ///     .expect("Failed to initialize ISO Date instance.");
    ///
    /// // Comparing dates: 2022-01-30 and 1992-09-02.
    /// let duration = newer_date_iso.until(
    ///     &date_iso,
    ///     DateDurationUnit::Years,
    ///     DateDurationUnit::Days,
    /// );
    /// assert_eq!(duration.years, 30);
    /// assert_eq!(duration.months, -8);
    /// assert_eq!(duration.days, 28);
    ///
    /// // Create new date with date advancement. Reassign to new variable.
    /// let mutated_date_iso = date_iso.added(DateDuration::new(1, 2, 3, 4));
    /// assert_eq!(mutated_date_iso.year().number, 1993);
    /// assert_eq!(mutated_date_iso.month().ordinal, 11);
    /// assert_eq!(mutated_date_iso.day_of_month().0, 27);
    /// ```
    #[derive(Copy, Clone, Eq, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type should be stable (and is intended to be constructed manually)
    pub struct DateDuration<C: Calendar + ?Sized> {
        /// The number of years
        pub years: i32,
        /// The number of months
        pub months: i32,
        /// The number of weeks
        pub weeks: i32,
        /// The number of days
        pub days: i32,
        /// A marker for the calendar
        pub marker: PhantomData<C>,
    }
    
    /// A "duration unit" used to specify the minimum or maximum duration of time to
    /// care about
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    #[allow(clippy::exhaustive_enums)] // this type should be stable
    pub enum DateDurationUnit {
        /// Duration in years
        Years,
        /// Duration in months
        Months,
        /// Duration in weeks
        Weeks,
        /// Duration in days
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
        /// Construct a DateDuration
        ///
        /// ```rust
        /// # use icu_calendar::*;
        /// // two years, three months, and five days
        /// let duration: DateDuration<Iso> = DateDuration::new(2, 3, 0, 5);
        /// ```
        pub fn new(years: i32, months: i32, weeks: i32, days: i32) -> Self {
            DateDuration {
                years,
                months,
                weeks,
                days,
                marker: PhantomData,
            }
        }
    
        /// Explicitly cast duration to one for a different calendar
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use displaydoc::Display;
use icu_provider::DataError;
use tinystr::{tinystr, TinyStr16, TinyStr4};
use writeable::Writeable;

#[cfg(feature = "std")]
impl std::error::Error for CalendarError {}

/// A list of error outcomes for various operations in the `icu_calendar` crate.
///
/// Re-exported as [`Error`](crate::Error).
#[derive(Display, Debug, Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum CalendarError {
    /// An input could not be parsed.
    #[displaydoc("Could not parse as integer")]
    Parse,
    /// An input overflowed its range.
    #[displaydoc("{field} must be between 0-{max}")]
    Overflow {
        /// The name of the field
        field: &'static str,
        /// The maximum value
        max: usize,
    },
    #[displaydoc("{field} must be between {min}-0")]
    /// An input underflowed its range.
    Underflow {
        /// The name of the field
        field: &'static str,
        /// The minimum value
        min: isize,
    },
    /// Out of range
    // TODO(Manishearth) turn this into a proper variant
    OutOfRange,
    /// Unknown era
    #[displaydoc("No era named {0} for calendar {1}")]
    UnknownEra(TinyStr16, &'static str),
    /// Unknown month code for a given calendar
    #[displaydoc("No month code named {0} for calendar {1}")]
    UnknownMonthCode(TinyStr4, &'static str),
    /// Missing required input field for formatting
    #[displaydoc("No value for {0}")]
    MissingInput(&'static str),
    /// No support for a given calendar in AnyCalendar
    #[displaydoc("AnyCalendar does not support calendar {0}")]
    UnknownAnyCalendarKind(TinyStr16),
    /// An operation required a calendar but a calendar was not provided.
    #[displaydoc("An operation required a calendar but a calendar was not provided")]
    MissingCalendar,
    /// An error originating inside of the [data provider](icu_provider).
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
    /// Create an error when an [`AnyCalendarKind`] is expected but not available.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu_calendar::AnyCalendarKind;
    /// use icu_calendar::CalendarError;
    ///
    /// let cal_str = "maori";
    ///
    /// AnyCalendarKind::get_for_bcp47_string(cal_str)
    ///     .ok_or_else(|| CalendarError::unknown_any_calendar_kind(cal_str))
    ///     .expect_err("Māori calendar is not yet supported");
    /// ```
    ///
    /// [`AnyCalendarKind`]: crate::AnyCalendarKind
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and implementations for the Ethiopian calendar.
//!
//! ```rust
//! use icu::calendar::{ethiopian::Ethiopian, Date, DateTime};
//!
//! // `Date` type
//! let date_iso = Date::try_new_iso_date(1970, 1, 2)
//!     .expect("Failed to initialize ISO Date instance.");
//! let date_ethiopian = Date::new_from_iso(date_iso, Ethiopian::new());
//!
//! // `DateTime` type
//! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
//!     .expect("Failed to initialize ISO DateTime instance.");
//! let datetime_ethiopian =
//!     DateTime::new_from_iso(datetime_iso, Ethiopian::new());
//!
//! // `Date` checks
//! assert_eq!(date_ethiopian.year().number, 1962);
//! assert_eq!(date_ethiopian.month().ordinal, 4);
//! assert_eq!(date_ethiopian.day_of_month().0, 24);
//!
//! // `DateTime` type
//! assert_eq!(datetime_ethiopian.date.year().number, 1962);
//! assert_eq!(datetime_ethiopian.date.month().ordinal, 4);
//! assert_eq!(datetime_ethiopian.date.day_of_month().0, 24);
//! assert_eq!(datetime_ethiopian.time.hour.number(), 13);
//! assert_eq!(datetime_ethiopian.time.minute.number(), 1);
//! assert_eq!(datetime_ethiopian.time.second.number(), 0);
//! ```

use crate::any_calendar::AnyCalendarKind;
use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
use crate::coptic::Coptic;
use crate::iso::Iso;
use crate::julian::Julian;
use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
use core::marker::PhantomData;
use tinystr::tinystr;

/// The number of years the Amete Alem epoch precedes the Amete Mihret epoch
const AMETE_ALEM_OFFSET: i32 = 5500;

/// Which era style the ethiopian calendar uses
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum EthiopianEraStyle {
    /// Use an era scheme of pre- and post- Incarnation eras,
    /// anchored at the date of the Incarnation of Jesus in this calendar
    AmeteMihret,
    /// Use an era scheme of the Anno Mundi era, anchored at the date of Creation
    /// in this calendar
    AmeteAlem,
}

/// The [Ethiopian Calendar]
///
/// The [Ethiopian calendar] is a solar calendar used by the Coptic Orthodox Church, with twelve normal months
/// and a thirteenth small epagomenal month.
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// It can be constructed in two modes: using the Amete Alem era scheme, or the Amete Mihret era scheme (the default),
/// see [`EthiopianEraStyle`] for more info.
///
/// [Ethiopian calendar]: https://en.wikipedia.org/wiki/Ethiopian_calendar
///
/// # Era codes
///
/// This calendar supports three era codes, based on what mode it is in. In the Amete Mihret scheme it has
/// the `"incar"` and `"pre-incar"` eras, 1 Incarnation is 9 CE. In the Amete Alem scheme, it instead has a single era,
/// `"mundi`, where 1 Anno Mundi is 5493 BCE. Dates before that use negative year numbers.
// The bool specifies whether dates should be in the Amete Alem era scheme
#[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
#[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
pub struct Ethiopian(pub(crate) bool);

/// The inner date type used for representing [`Date`]s of [`Ethiopian`]. See [`Date`] and [`Ethiopian`] for more details.
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
    /// Construct a new Ethiopian Calendar for the Amete Mihret era naming scheme
    pub fn new() -> Self {
        Self(false)
    }
    /// Construct a new Ethiopian Calendar with a value specifying whether or not it is Amete Alem
    pub fn new_with_era_style(era_style: EthiopianEraStyle) -> Self {
        Self(era_style == EthiopianEraStyle::AmeteAlem)
    }
    /// Set whether or not this uses the Amete Alem era scheme
    pub fn set_era_style(&mut self, era_style: EthiopianEraStyle) {
        self.0 = era_style == EthiopianEraStyle::AmeteAlem
    }

    /// Returns whether this has the Amete Alem era
    pub fn era_style(&self) -> EthiopianEraStyle {
        if self.0 {
            EthiopianEraStyle::AmeteAlem
        } else {
            EthiopianEraStyle::AmeteMihret
        }
    }

    // "Fixed" is a day count representation of calendars staring from Jan 1st of year 1 of the Georgian Calendar.
    // The fixed date algorithms are from
    // Dershowitz, Nachum, and Edward M. Reingold. _Calendrical calculations_. Cambridge University Press, 2008.
    //
    // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L2017
    fn fixed_from_ethiopian(date: ArithmeticDate<Ethiopian>) -> i32 {
        Coptic::fixed_from_coptic_integers(date.year, date.month, date.day)
            - ETHIOPIC_TO_COPTIC_OFFSET
    }

    // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L2028
    fn ethiopian_from_fixed(date: i32) -> EthiopianDateInner {
        let coptic_date = Coptic::coptic_from_fixed(date + ETHIOPIC_TO_COPTIC_OFFSET);

        #[allow(clippy::unwrap_used)] // Coptic and Ethiopic have the same allowed ranges for dates
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
    /// Construct new Ethiopian Date.
    ///
    /// For the Amete Mihret era style, negative years work with
    /// year 0 as 1 pre-Incarnation, year -1 as 2 pre-Incarnation,
    /// and so on.
    ///
    /// ```rust
    /// use icu::calendar::ethiopian::EthiopianEraStyle;
    /// use icu::calendar::Date;
    ///
    /// let date_ethiopian = Date::try_new_ethiopian_date(
    ///     EthiopianEraStyle::AmeteMihret,
    ///     2014,
    ///     8,
    ///     25,
    /// )
    /// .expect("Failed to initialize Ethopic Date instance.");
    ///
    /// assert_eq!(date_ethiopian.year().number, 2014);
    /// assert_eq!(date_ethiopian.month().ordinal, 8);
    /// assert_eq!(date_ethiopian.day_of_month().0, 25);
    /// ```
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
    /// Construct a new Ethiopian datetime from integers.
    ///
    /// For the Amete Mihret era style, negative years work with
    /// year 0 as 1 pre-Incarnation, year -1 as 2 pre-Incarnation,
    /// and so on.
    ///
    /// ```rust
    /// use icu::calendar::ethiopian::EthiopianEraStyle;
    /// use icu::calendar::DateTime;
    ///
    /// let datetime_ethiopian = DateTime::try_new_ethiopian_datetime(
    ///     EthiopianEraStyle::AmeteMihret,
    ///     2014,
    ///     8,
    ///     25,
    ///     13,
    ///     1,
    ///     0,
    /// )
    /// .expect("Failed to initialize Ethiopian DateTime instance.");
    ///
    /// assert_eq!(datetime_ethiopian.date.year().number, 2014);
    /// assert_eq!(datetime_ethiopian.date.month().ordinal, 8);
    /// assert_eq!(datetime_ethiopian.date.day_of_month().0, 25);
    /// assert_eq!(datetime_ethiopian.time.hour.number(), 13);
    /// assert_eq!(datetime_ethiopian.time.minute.number(), 1);
    /// assert_eq!(datetime_ethiopian.time.second.number(), 0);
    /// ```
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
        // 11th September 2023 in gregorian is 6/13/2015 in ethiopian
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
        // https://github.com/unicode-org/icu4x/issues/2254
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
            let time = match DateTime::try_new_iso_datetime(year, month, day, hour, minute, second) {
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and implementations for the Gregorian calendar.
//!
//! ```rust
//! use icu::calendar::{gregorian::Gregorian, Date, DateTime};
//!
//! // `Date` type
//! let date_iso = Date::try_new_iso_date(1970, 1, 2)
//!     .expect("Failed to initialize ISO Date instance.");
//! let date_gregorian = Date::new_from_iso(date_iso, Gregorian);
//!
//! // `DateTime` type
//! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
//!     .expect("Failed to initialize ISO DateTime instance.");
//! let datetime_gregorian = DateTime::new_from_iso(datetime_iso, Gregorian);
//!
//! // `Date` checks
//! assert_eq!(date_gregorian.year().number, 1970);
//! assert_eq!(date_gregorian.month().ordinal, 1);
//! assert_eq!(date_gregorian.day_of_month().0, 2);
//!
//! // `DateTime` type
//! assert_eq!(datetime_gregorian.date.year().number, 1970);
//! assert_eq!(datetime_gregorian.date.month().ordinal, 1);
//! assert_eq!(datetime_gregorian.date.day_of_month().0, 2);
//! assert_eq!(datetime_gregorian.time.hour.number(), 13);
//! assert_eq!(datetime_gregorian.time.minute.number(), 1);
//! assert_eq!(datetime_gregorian.time.second.number(), 0);
//! ```

use crate::any_calendar::AnyCalendarKind;
use crate::calendar_arithmetic::ArithmeticDate;
use crate::iso::{Iso, IsoDateInner};
use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
use tinystr::tinystr;

/// The Gregorian Calendar
///
/// The [Gregorian calendar] is a solar calendar used by most of the world, with twelve months.
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// [Gregorian calendar]: https://en.wikipedia.org/wiki/Gregorian_calendar
///
/// # Era codes
///
/// This calendar supports two era codes: `"bce"`, and `"ce"`, corresponding to the BCE and CE eras
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
#[allow(clippy::exhaustive_structs)] // this type is stable
pub struct Gregorian;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
/// The inner date type used for representing [`Date`]s of [`Gregorian`]. See [`Date`] and [`Gregorian`] for more details.
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

    /// The calendar-specific year represented by `date`
    fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
        year_as_gregorian(date.0 .0.year)
    }

    /// The calendar-specific month represented by `date`
    fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
        Iso.month(&date.0)
    }

    /// The calendar-specific day-of-month represented by `date`
    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        Iso.day_of_month(&date.0)
    }

    /// Information of the day of the year
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
    /// Construct a new Gregorian Date.
    ///
    /// Years are specified as ISO years.
    ///
    /// ```rust
    /// use icu::calendar::Date;
    /// use std::convert::TryFrom;
    ///
    /// // Conversion from ISO to Gregorian
    /// let date_gregorian = Date::try_new_gregorian_date(1970, 1, 2)
    ///     .expect("Failed to initialize Gregorian Date instance.");
    ///
    /// assert_eq!(date_gregorian.year().number, 1970);
    /// assert_eq!(date_gregorian.month().ordinal, 1);
    /// assert_eq!(date_gregorian.day_of_month().0, 2);
    /// ```
    pub fn try_new_gregorian_date(
        year: i32,
        month: u8,
        day: u8,
    ) -> Result<Date<Gregorian>, CalendarError> {
        Date::try_new_iso_date(year, month, day).map(|d| Date::new_from_iso(d, Gregorian))
    }
}

impl DateTime<Gregorian> {
    /// Construct a new Gregorian datetime from integers.
    ///
    /// Years are specified as ISO years.
    ///
    /// ```rust
    /// use icu::calendar::DateTime;
    ///
    /// let datetime_gregorian =
    ///     DateTime::try_new_gregorian_datetime(1970, 1, 2, 13, 1, 0)
    ///         .expect("Failed to initialize Gregorian DateTime instance.");
    ///
    /// assert_eq!(datetime_gregorian.date.year().number, 1970);
    /// assert_eq!(datetime_gregorian.date.month().ordinal, 1);
    /// assert_eq!(datetime_gregorian.date.day_of_month().0, 2);
    /// assert_eq!(datetime_gregorian.time.hour.number(), 13);
    /// assert_eq!(datetime_gregorian.time.minute.number(), 1);
    /// assert_eq!(datetime_gregorian.time.second.number(), 0);
    /// ```
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

/// Calculate `(n / d, n % d)` such that the remainder is always positive.
pub fn div_rem_euclid(n: i32, d: i32) -> (i32, i32) {
    debug_assert!(d > 0);
    let (a, b) = (n / d, n % d);
    if n >= 0 || b == 0 {
        (a, b)
    } else {
        (a - 1, d + b)
    }
}

/// Calculate `n / d` such that the remainder is always positive.
/// This is equivalent to quotient() in the Reingold/Dershowitz Lisp code
pub const fn quotient(n: i32, d: i32) -> i32 {
    debug_assert!(d > 0);
    // Code can use int_roundings once stabilized
    // https://github.com/rust-lang/rust/issues/88581
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and implementations for the Indian national calendar.
//!
//! ```rust
//! use icu::calendar::{indian::Indian, Date, DateTime};
//!
//! // `Date` type
//! let date_iso = Date::try_new_iso_date(1970, 1, 2)
//!     .expect("Failed to initialize ISO Date instance.");
//! let date_indian = Date::new_from_iso(date_iso, Indian);
//!
//! // `DateTime` type
//! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
//!     .expect("Failed to initialize ISO DateTime instance.");
//! let datetime_indian = DateTime::new_from_iso(datetime_iso, Indian);
//!
//! // `Date` checks
//! assert_eq!(date_indian.year().number, 1891);
//! assert_eq!(date_indian.month().ordinal, 10);
//! assert_eq!(date_indian.day_of_month().0, 12);
//!
//! // `DateTime` type
//! assert_eq!(datetime_indian.date.year().number, 1891);
//! assert_eq!(datetime_indian.date.month().ordinal, 10);
//! assert_eq!(datetime_indian.date.day_of_month().0, 12);
//! assert_eq!(datetime_indian.time.hour.number(), 13);
//! assert_eq!(datetime_indian.time.minute.number(), 1);
//! assert_eq!(datetime_indian.time.second.number(), 0);
//! ```

use crate::any_calendar::AnyCalendarKind;
use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
use crate::iso::Iso;
use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
use core::marker::PhantomData;
use tinystr::tinystr;

/// The Indian National Calendar (aka the Saka calendar)
///
/// The [Indian National calendar] is a solar calendar used by the Indian government, with twelve months.
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// [Indian National calendar]: https://en.wikipedia.org/wiki/Indian_national_calendar
///
/// # Era codes
///
/// This calendar has a single era: `"saka"`, with Saka 0 being 78 CE. Dates before this era use negative years.
#[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
#[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
#[allow(clippy::exhaustive_structs)] // this type is stable
pub struct Indian;

/// The inner date type used for representing [`Date`]s of [`Indian`]. See [`Date`] and [`Indian`] for more details.
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

/// The Saka calendar starts on the 81st day of the Gregorian year (March 22 or 21)
/// which is an 80 day offset. This number should be subtracted from Gregorian dates
const DAY_OFFSET: u32 = 80;
/// The Saka calendar is 78 years behind Gregorian. This number should be added to Gregorian dates
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

    //
    fn date_from_iso(&self, iso: Date<Iso>) -> IndianDateInner {
        // Get day number in year (1 indexed)
        let day_of_year_iso = Iso::day_of_year(*iso.inner());
        // Convert to Saka year
        let mut year = iso.inner().0.year - YEAR_OFFSET;
        // This is in the previous Indian year
        let day_of_year_indian = if day_of_year_iso <= DAY_OFFSET {
            year -= 1;
            let n_days = Self::days_in_provided_year(year);

            // calculate day of year in previous year
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
            // calculate day of year in next year
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
    /// Construct a new Indian Calendar
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
    /// Construct new Indian Date, with year provided in the Śaka era.
    ///
    /// ```rust
    /// use icu::calendar::Date;
    ///
    /// let date_indian = Date::try_new_indian_date(1891, 10, 12)
    ///     .expect("Failed to initialize Indian Date instance.");
    ///
    /// assert_eq!(date_indian.year().number, 1891);
    /// assert_eq!(date_indian.month().ordinal, 10);
    /// assert_eq!(date_indian.day_of_month().0, 12);
    /// ```
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
    /// Construct a new Indian datetime from integers, with year provided in the Śaka era.
    ///
    /// ```rust
    /// use icu::calendar::DateTime;
    ///
    /// let datetime_indian =
    ///     DateTime::try_new_indian_datetime(1891, 10, 12, 13, 1, 0)
    ///         .expect("Failed to initialize Indian DateTime instance.");
    ///
    /// assert_eq!(datetime_indian.date.year().number, 1891);
    /// assert_eq!(datetime_indian.date.month().ordinal, 10);
    /// assert_eq!(datetime_indian.date.day_of_month().0, 12);
    /// assert_eq!(datetime_indian.time.hour.number(), 13);
    /// assert_eq!(datetime_indian.time.minute.number(), 1);
    /// assert_eq!(datetime_indian.time.second.number(), 0);
    /// ```
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
        let indian =
            Date::try_new_indian_date(y, m, d).expect("Indian date should construct successfully");
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
        // Ultimately the day of the year will always be identical regardless of it
        // being a leap year or not
        // Test dates that occur after and before Chaitra 1 (March 22/21), in all years of
        // a four-year leap cycle, to ensure that all code paths are tested
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
pub mod iso {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    //! This module contains types and implementations for the ISO calendar.
    //!
    //! ```rust
    //! use icu::calendar::{Date, DateTime};
    //!
    //! // `Date` type
    //! let date_iso = Date::try_new_iso_date(1970, 1, 2)
    //!     .expect("Failed to initialize ISO Date instance.");
    //!
    //! // `DateTime` type
    //! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
    //!     .expect("Failed to initialize ISO DateTime instance.");
    //!
    //! // `Date` checks
    //! assert_eq!(date_iso.year().number, 1970);
    //! assert_eq!(date_iso.month().ordinal, 1);
    //! assert_eq!(date_iso.day_of_month().0, 2);
    //!
    //! // `DateTime` type
    //! assert_eq!(datetime_iso.date.year().number, 1970);
    //! assert_eq!(datetime_iso.date.month().ordinal, 1);
    //! assert_eq!(datetime_iso.date.day_of_month().0, 2);
    //! assert_eq!(datetime_iso.time.hour.number(), 13);
    //! assert_eq!(datetime_iso.time.minute.number(), 1);
    //! assert_eq!(datetime_iso.time.second.number(), 0);
    //! ```
    
    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::{div_rem_euclid, quotient};
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use tinystr::tinystr;
    
    // The georgian epoch is equivalent to first day in fixed day measurement
    const EPOCH: i32 = 1;
    
    /// The [ISO Calendar]
    ///
    /// The [ISO Calendar] is a standardized solar calendar with twelve months.
    /// It is identical to the Gregorian calendar, except it uses negative years for years before 1 CE,
    /// and may have differing formatting data for a given locale.
    ///
    /// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
    ///
    /// [ISO Calendar]: https://en.wikipedia.org/wiki/ISO_calendar
    ///
    /// # Era codes
    ///
    /// This calendar supports one era, `"default"`
    
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Iso;
    
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    /// The inner date type used for representing [`Date`]s of [`Iso`]. See [`Date`] and [`Iso`] for more details.
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
        /// Construct a date from era/month codes and fields
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
            // For the purposes of the calculation here, Monday is 0, Sunday is 6
            // ISO has Monday=1, Sunday=7, which we transform in the last step
    
            // The days of the week are the same every 400 years
            // so we normalize to the nearest multiple of 400
            let years_since_400 = date.0.year % 400;
            let leap_years_since_400 = years_since_400 / 4 - years_since_400 / 100;
            // The number of days to the current year
            let days_to_current_year = 365 * years_since_400 + leap_years_since_400;
            // The weekday offset from January 1 this year and January 1 2000
            let year_offset = days_to_current_year % 7;
    
            // Corresponding months from
            // https://en.wikipedia.org/wiki/Determination_of_the_day_of_the_week#Corresponding_months
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
    
            // We calculated in a zero-indexed fashion, but ISO specifies one-indexed
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
    
        /// The calendar-specific year represented by `date`
        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            Self::year_as_iso(date.0.year)
        }
    
        /// The calendar-specific month represented by `date`
        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }
    
        /// The calendar-specific day-of-month represented by `date`
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
        /// Construct a new ISO date from integers.
        ///
        /// ```rust
        /// use icu::calendar::Date;
        ///
        /// let date_iso = Date::try_new_iso_date(1970, 1, 2)
        ///     .expect("Failed to initialize ISO Date instance.");
        ///
        /// assert_eq!(date_iso.year().number, 1970);
        /// assert_eq!(date_iso.month().ordinal, 1);
        /// assert_eq!(date_iso.day_of_month().0, 2);
        /// ```
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
        /// Construct a new ISO datetime from integers.
        ///
        /// ```rust
        /// use icu::calendar::DateTime;
        ///
        /// let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
        ///     .expect("Failed to initialize ISO DateTime instance.");
        ///
        /// assert_eq!(datetime_iso.date.year().number, 1970);
        /// assert_eq!(datetime_iso.date.month().ordinal, 1);
        /// assert_eq!(datetime_iso.date.day_of_month().0, 2);
        /// assert_eq!(datetime_iso.time.hour.number(), 13);
        /// assert_eq!(datetime_iso.time.minute.number(), 1);
        /// assert_eq!(datetime_iso.time.second.number(), 0);
        /// ```
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
    
        /// Minute count representation of calendars starting from 00:00:00 on Jan 1st, 1970.
        ///
        /// ```rust
        /// use icu::calendar::DateTime;
        ///
        /// let today = DateTime::try_new_iso_datetime(2020, 2, 29, 0, 0, 0).unwrap();
        ///
        /// assert_eq!(today.minutes_since_local_unix_epoch(), 26382240);
        /// assert_eq!(
        ///     DateTime::from_minutes_since_local_unix_epoch(26382240),
        ///     today
        /// );
        ///
        /// let today = DateTime::try_new_iso_datetime(1970, 1, 1, 0, 0, 0).unwrap();
        ///
        /// assert_eq!(today.minutes_since_local_unix_epoch(), 0);
        /// assert_eq!(DateTime::from_minutes_since_local_unix_epoch(0), today);
        /// ```
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
    
        /// Convert minute count since 00:00:00 on Jan 1st, 1970 to ISO Date.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use icu::calendar::DateTime;
        ///
        /// // After Unix Epoch
        /// let today = DateTime::try_new_iso_datetime(2020, 2, 29, 0, 0, 0).unwrap();
        ///
        /// assert_eq!(today.minutes_since_local_unix_epoch(), 26382240);
        /// assert_eq!(
        ///     DateTime::from_minutes_since_local_unix_epoch(26382240),
        ///     today
        /// );
        ///
        /// // Unix Epoch
        /// let today = DateTime::try_new_iso_datetime(1970, 1, 1, 0, 0, 0).unwrap();
        ///
        /// assert_eq!(today.minutes_since_local_unix_epoch(), 0);
        /// assert_eq!(DateTime::from_minutes_since_local_unix_epoch(0), today);
        ///
        /// // Before Unix Epoch
        /// let today = DateTime::try_new_iso_datetime(1967, 4, 6, 20, 40, 0).unwrap();
        ///
        /// assert_eq!(today.minutes_since_local_unix_epoch(), -1440200);
        /// assert_eq!(
        ///     DateTime::from_minutes_since_local_unix_epoch(-1440200),
        ///     today
        /// );
        /// ```
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
        /// Construct a new ISO Calendar
        pub fn new() -> Self {
            Self
        }
    
        /// Count the number of days in a given month/year combo
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
    
        // Fixed is day count representation of calendars starting from Jan 1st of year 1.
        // The fixed calculations algorithms are from the Calendrical Calculations book.
        //
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1167-L1189
        pub(crate) fn fixed_from_iso(date: IsoDateInner) -> i32 {
            // Calculate days per year
            let mut fixed: i32 = EPOCH - 1 + 365 * (date.0.year - 1);
            // Adjust for leap year logic
            fixed += quotient(date.0.year - 1, 4) - quotient(date.0.year - 1, 100)
                + quotient(date.0.year - 1, 400);
            // Days of current year
            fixed += quotient(367 * (date.0.month as i32) - 362, 12);
            // Leap year adjustment for the current year
            fixed += if date.0.month <= 2 {
                0
            } else if Self::is_leap_year(date.0.year) {
                -1
            } else {
                -2
            };
            // Days passed in current month
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
    
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1191-L1217
        fn iso_year_from_fixed(date: i32) -> i32 {
            let date = date - EPOCH;
            // 400 year cycles have 146097 days
            let (n_400, date) = div_rem_euclid(date, 146097);
    
            // 100 year cycles have 36524 days
            let (n_100, date) = div_rem_euclid(date, 36524);
    
            // 4 year cycles have 1461 days
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
    
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1237-L1258
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
            // Cumulatively how much are dates in each month
            // offset from "30 days in each month" (in non leap years)
            let month_offset = [0, 1, -1, 0, 0, 1, 1, 2, 3, 3, 4, 4];
            #[allow(clippy::indexing_slicing)] // date.0.month in 1..=12
            let mut offset = month_offset[date.0.month as usize - 1];
            if Self::is_leap_year(date.0.year) && date.0.month > 2 {
                // Months after February in a leap year are offset by one less
                offset += 1;
            }
            let prev_month_days = (30 * (date.0.month as i32 - 1) + offset) as u32;
    
            prev_month_days + date.0.day as u32
        }
    
        /// Wrap the year in the appropriate era code
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
            // June 23, 2021 is a Wednesday
            assert_eq!(
                Date::try_new_iso_date(2021, 6, 23).unwrap().day_of_week(),
                IsoWeekday::Wednesday,
            );
            // Feb 2, 1983 was a Wednesday
            assert_eq!(
                Date::try_new_iso_date(1983, 2, 2).unwrap().day_of_week(),
                IsoWeekday::Wednesday,
            );
            // Jan 21, 2021 was a Tuesday
            assert_eq!(
                Date::try_new_iso_date(2020, 1, 21).unwrap().day_of_week(),
                IsoWeekday::Tuesday,
            );
        }
    
        #[test]
        fn test_day_of_year() {
            // June 23, 2021 was day 174
            assert_eq!(
                Date::try_new_iso_date(2021, 6, 23)
                    .unwrap()
                    .day_of_year_info()
                    .day_of_year,
                174,
            );
            // June 23, 2020 was day 175
            assert_eq!(
                Date::try_new_iso_date(2020, 6, 23)
                    .unwrap()
                    .day_of_year_info()
                    .day_of_year,
                175,
            );
            // Feb 2, 1983 was a Wednesday
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
            // since 2021/02/31 isn't a valid date, `offset_date` auto-adjusts by adding 3 days to 2021/02/28
            let today_plus_1_month = Date::try_new_iso_date(2021, 3, 3).unwrap();
            let offset = today.added(DateDuration::new(0, 1, 0, 0));
            assert_eq!(offset, today_plus_1_month);
    
            let today = Date::try_new_iso_date(2021, 1, 31).unwrap();
            // since 2021/02/31 isn't a valid date, `offset_date` auto-adjusts by adding 3 days to 2021/02/28
            let today_plus_1_month_1_day = Date::try_new_iso_date(2021, 3, 4).unwrap();
            let offset = today.added(DateDuration::new(0, 1, 0, 1));
            assert_eq!(offset, today_plus_1_month_1_day);
        }
    
        #[test]
        fn test_iso_to_from_fixed() {
            // Reminder: ISO year 0 is Gregorian year 1 BCE.
            // Year 0 is a leap year due to the 400-year rule.
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
                let today = DateTime::try_new_iso_datetime(year, month, day, hour, minute, 0).unwrap();
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
    // This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This module contains types and implementations for the Japanese calendar.
//!
//! ```rust
//! use icu::calendar::japanese::Japanese;
//! use icu::calendar::{types::Era, Date, DateTime};
//! use tinystr::tinystr;
//!
//! // `icu_testdata::unstable` contains information specifying era dates.
//! // Production code should probably use its own data provider
//! let japanese_calendar =
//!     Japanese::try_new_unstable(&icu_testdata::unstable())
//!         .expect("Cannot load japanese data");
//!
//! // `Date` type
//! let date_iso = Date::try_new_iso_date(1970, 1, 2)
//!     .expect("Failed to initialize ISO Date instance.");
//! let date_japanese = Date::new_from_iso(date_iso, japanese_calendar.clone());
//!
//! // `DateTime` type
//! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
//!     .expect("Failed to initialize ISO DateTime instance.");
//! let datetime_japanese =
//!     DateTime::new_from_iso(datetime_iso, japanese_calendar.clone());
//!
//! // `Date` checks
//! assert_eq!(date_japanese.year().number, 45);
//! assert_eq!(date_japanese.month().ordinal, 1);
//! assert_eq!(date_japanese.day_of_month().0, 2);
//! assert_eq!(date_japanese.year().era, Era(tinystr!(16, "showa")));
//!
//! // `DateTime` type
//! assert_eq!(datetime_japanese.date.year().number, 45);
//! assert_eq!(datetime_japanese.date.month().ordinal, 1);
//! assert_eq!(datetime_japanese.date.day_of_month().0, 2);
//! assert_eq!(
//!     datetime_japanese.date.year().era,
//!     Era(tinystr!(16, "showa"))
//! );
//! assert_eq!(datetime_japanese.time.hour.number(), 13);
//! assert_eq!(datetime_japanese.time.minute.number(), 1);
//! assert_eq!(datetime_japanese.time.second.number(), 0);
//! ```

use crate::any_calendar::AnyCalendarKind;
use crate::iso::{Iso, IsoDateInner};
use crate::provider::{EraStartDate, JapaneseErasV1Marker, JapaneseExtendedErasV1Marker};
use crate::{
    types, AsCalendar, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime, Ref,
};
use icu_provider::prelude::*;
use tinystr::{tinystr, TinyStr16};

/// The [Japanese Calendar] (with modern eras only)
///
/// The [Japanese calendar] is a solar calendar used in Japan, with twelve months.
/// The months and days are identical to that of the Gregorian calendar, however the years are counted
/// differently using the Japanese era system.
///
/// This calendar only contains eras after Meiji, for all historical eras, check out [`JapaneseExtended`].
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// [Japanese calendar]: https://en.wikipedia.org/wiki/Japanese_calendar
///
/// # Era codes
///
/// This calendar currently supports seven era codes. It supports the five post-Meiji eras
/// (`"meiji"`, `"taisho"`, `"showa"`, `"heisei"`, `"reiwa"`), as well as using the Gregorian
/// `"bce"` and `"ce"` for dates before the Meiji era.
///
/// Future eras will also be added to this type when they are decided.
///
/// These eras are loaded from data, requiring a data provider capable of providing [`JapaneseErasV1Marker`]
/// data (`calendar/japanese@1`).
#[derive(Clone, Debug, Default)]
pub struct Japanese {
    eras: DataPayload<JapaneseErasV1Marker>,
}

/// The [Japanese Calendar] (with historical eras)
///
/// The [Japanese calendar] is a solar calendar used in Japan, with twelve months.
/// The months and days are identical to that of the Gregorian calendar, however the years are counted
/// differently using the Japanese era system.
///
/// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
///
/// [Japanese calendar]: https://en.wikipedia.org/wiki/Japanese_calendar
///
/// # Era codes
///
/// This calendar supports a large number of era codes. It supports the five post-Meiji eras
/// (`"meiji"`, `"taisho"`, `"showa"`, `"heisei"`, `"reiwa"`). Pre-Meiji eras are represented
/// with their names converted to lowercase ascii and followed by their start year. E.g. the "Ten'ō"
/// era (781 - 782 CE) has the code `"teno-781"`. The  Gregorian `"bce"` and `"ce"` eras
/// are used for dates before the first known era era.
///
///
/// These eras are loaded from data, requiring a data provider capable of providing [`JapaneseExtendedErasV1Marker`]
/// data (`calendar/japanext@1`).
#[derive(Clone, Debug, Default)]
pub struct JapaneseExtended(Japanese);

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
/// The inner date type used for representing [`Date`]s of [`Japanese`]. See [`Date`] and [`Japanese`] for more details.
pub struct JapaneseDateInner {
    inner: IsoDateInner,
    adjusted_year: i32,
    era: TinyStr16,
}

impl Japanese {
    /// Creates a new [`Japanese`] from locale data using only modern eras (post-meiji).
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    /// <div class="stab unstable">
    /// ⚠️ The bounds on this function may change over time, including in SemVer minor releases.
    /// </div>
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

    icu_provider::gen_any_buffer_constructors!(locale: skip, options: skip, error: CalendarError);

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
    /// Creates a new [`Japanese`] from locale data using all eras (including pre-meiji).
    ///
    /// [📚 Help choosing a constructor](icu_provider::constructors)
    /// <div class="stab unstable">
    /// ⚠️ The bounds on this function may change over time, including in SemVer minor releases.
    /// </div>
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

    icu_provider::gen_any_buffer_constructors!(locale: skip, options: skip, error: CalendarError);
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

    /// The calendar-specific year represented by `date`
    fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
        types::FormattableYear {
            era: types::Era(date.era),
            number: date.adjusted_year,
            related_iso: None,
        }
    }

    /// The calendar-specific month represented by `date`
    fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
        Iso.month(&date.inner)
    }

    /// The calendar-specific day-of-month represented by `date`
    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        Iso.day_of_month(&date.inner)
    }

    /// Information of the day of the year
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

    /// The calendar-specific year represented by `date`
    fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
        Japanese::year(&self.0, date)
    }

    /// The calendar-specific month represented by `date`
    fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
        Japanese::month(&self.0, date)
    }

    /// The calendar-specific day-of-month represented by `date`
    fn day_of_month(&self, date: &Self::DateInner) -> types::DayOfMonth {
        Japanese::day_of_month(&self.0, date)
    }

    /// Information of the day of the year
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
    /// Construct a new Japanese Date.
    ///
    /// Years are specified in the era provided, and must be in range for Japanese
    /// eras (e.g. dates past April 30 Heisei 31 must be in Reiwa; "Jun 5 Heisei 31" and "Jan 1 Heisei 32"
    /// will not be adjusted to being in Reiwa 1 and 2 respectively)
    ///
    /// However, dates may always be specified in "bce" or "ce" and they will be adjusted as necessary.
    ///
    /// ```rust
    /// use icu::calendar::japanese::Japanese;
    /// use icu::calendar::{types, Date, Ref};
    /// use std::convert::TryFrom;
    /// use tinystr::tinystr;
    ///
    /// let japanese_calendar =
    ///     Japanese::try_new_unstable(&icu_testdata::unstable())
    ///         .expect("Cannot load japanese data");
    /// // for easy sharing
    /// let japanese_calendar = Ref(&japanese_calendar);
    ///
    /// let era = types::Era(tinystr!(16, "heisei"));
    ///
    /// let date = Date::try_new_japanese_date(era, 14, 1, 2, japanese_calendar)
    ///     .expect("Constructing a date should succeed");
    ///
    /// assert_eq!(date.year().era, era);
    /// assert_eq!(date.year().number, 14);
    /// assert_eq!(date.month().ordinal, 1);
    /// assert_eq!(date.day_of_month().0, 2);
    ///
    /// // This function will error for eras that are out of bounds:
    /// // (Heisei was 32 years long, Heisei 33 is in Reiwa)
    /// let oob_date =
    ///     Date::try_new_japanese_date(era, 33, 1, 2, japanese_calendar);
    /// assert!(oob_date.is_err());
    ///
    /// // and for unknown eras
    /// let fake_era = types::Era(tinystr!(16, "neko")); // 🐱
    /// let fake_date =
    ///     Date::try_new_japanese_date(fake_era, 10, 1, 2, japanese_calendar);
    /// assert!(fake_date.is_err());
    /// ```
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
    /// Construct a new Japanese Date with all eras.
    ///
    /// Years are specified in the era provided, and must be in range for Japanese
    /// eras (e.g. dates past April 30 Heisei 31 must be in Reiwa; "Jun 5 Heisei 31" and "Jan 1 Heisei 32"
    /// will not be adjusted to being in Reiwa 1 and 2 respectively)
    ///
    /// However, dates may always be specified in "bce" or "ce" and they will be adjusted as necessary.
    ///
    /// ```rust
    /// use icu::calendar::japanese::JapaneseExtended;
    /// use icu::calendar::{types, Date, Ref};
    /// use std::convert::TryFrom;
    /// use tinystr::tinystr;
    ///
    /// let japanext_calendar =
    ///     JapaneseExtended::try_new_unstable(&icu_testdata::unstable())
    ///         .expect("Cannot load japanese data");
    /// // for easy sharing
    /// let japanext_calendar = Ref(&japanext_calendar);
    ///
    /// let era = types::Era(tinystr!(16, "kansei-1789"));
    ///
    /// let date =
    ///     Date::try_new_japanese_extended_date(era, 7, 1, 2, japanext_calendar)
    ///         .expect("Constructing a date should succeed");
    ///
    /// assert_eq!(date.year().era, era);
    /// assert_eq!(date.year().number, 7);
    /// assert_eq!(date.month().ordinal, 1);
    /// assert_eq!(date.day_of_month().0, 2);
    /// ```
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

    /// For testing era fallback in icu_datetime
    #[doc(hidden)]
    pub fn into_japanese_date(self) -> Date<Japanese> {
        Date::from_raw(self.inner, self.calendar.0)
    }
}

impl DateTime<Japanese> {
    /// Construct a new Japanese datetime from integers.
    ///
    /// Years are specified in the era provided.
    ///
    /// ```rust
    /// use icu::calendar::japanese::Japanese;
    /// use icu::calendar::{types, DateTime};
    /// use std::convert::TryFrom;
    /// use tinystr::tinystr;
    ///
    /// let japanese_calendar =
    ///     Japanese::try_new_unstable(&icu_testdata::unstable())
    ///         .expect("Cannot load japanese data");
    ///
    /// let era = types::Era(tinystr!(16, "heisei"));
    ///
    /// let datetime = DateTime::try_new_japanese_datetime(
    ///     era,
    ///     14,
    ///     1,
    ///     2,
    ///     13,
    ///     1,
    ///     0,
    ///     japanese_calendar,
    /// )
    /// .expect("Constructing a date should succeed");
    ///
    /// assert_eq!(datetime.date.year().era, era);
    /// assert_eq!(datetime.date.year().number, 14);
    /// assert_eq!(datetime.date.month().ordinal, 1);
    /// assert_eq!(datetime.date.day_of_month().0, 2);
    /// assert_eq!(datetime.time.hour.number(), 13);
    /// assert_eq!(datetime.time.minute.number(), 1);
    /// assert_eq!(datetime.time.second.number(), 0);
    /// ```
    #[allow(clippy::too_many_arguments)] // it's more convenient to have this many arguments
                                         // if people wish to construct this by parts they can use
                                         // Date::try_new_japanese_date() + DateTime::new(date, time)
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
    /// Construct a new Japanese datetime from integers with all eras.
    ///
    /// Years are specified in the era provided.
    ///
    /// ```rust
    /// use icu::calendar::japanese::JapaneseExtended;
    /// use icu::calendar::{types, DateTime};
    /// use std::convert::TryFrom;
    /// use tinystr::tinystr;
    ///
    /// let japanext_calendar =
    ///     JapaneseExtended::try_new_unstable(&icu_testdata::unstable())
    ///         .expect("Cannot load japanese data");
    ///
    /// let era = types::Era(tinystr!(16, "kansei-1789"));
    ///
    /// let datetime = DateTime::try_new_japanese_extended_datetime(
    ///     era,
    ///     7,
    ///     1,
    ///     2,
    ///     13,
    ///     1,
    ///     0,
    ///     japanext_calendar,
    /// )
    /// .expect("Constructing a date should succeed");
    ///
    /// assert_eq!(datetime.date.year().era, era);
    /// assert_eq!(datetime.date.year().number, 7);
    /// assert_eq!(datetime.date.month().ordinal, 1);
    /// assert_eq!(datetime.date.day_of_month().0, 2);
    /// assert_eq!(datetime.time.hour.number(), 13);
    /// assert_eq!(datetime.time.minute.number(), 1);
    /// assert_eq!(datetime.time.second.number(), 0);
    /// ```
    #[allow(clippy::too_many_arguments)] // it's more convenient to have this many arguments
                                         // if people wish to construct this by parts they can use
                                         // Date::try_new_japanese_date() + DateTime::new(date, time)
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
            date: Date::try_new_japanese_extended_date(era, year, month, day, japanext_calendar)?,
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
    /// Given an ISO date, give year and era for that date in the Japanese calendar
    ///
    /// This will also use Gregorian eras for eras that are before the earliest era
    fn adjusted_year_for(&self, date: &IsoDateInner) -> (i32, TinyStr16) {
        let date: EraStartDate = date.into();
        let (start, era) = self.japanese_era_for(date);
        // The year in which an era starts is Year 1, and it may be short
        // The only time this function will experience dates that are *before*
        // the era start date are for the first era (Currently, taika-645
        // for japanext, meiji for japanese),
        // In such a case, we instead fall back to Gregorian era codes
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

    /// Given an date, obtain the era data (not counting spliced gregorian eras)
    fn japanese_era_for(&self, date: EraStartDate) -> (EraStartDate, TinyStr16) {
        let era_data = self.eras.get();
        // We optimize for the five "modern" post-Meiji eras, which are stored in a smaller
        // array and also hardcoded. The hardcoded version is not used if data indicates the
        // presence of newer eras.
        if date >= MEIJI_START
            && era_data.dates_to_eras.last().map(|x| x.1) == Some(tinystr!(16, "reiwa"))
        {
            // Fast path in case eras have not changed since this code was written
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

    /// Returns the range of dates for a given Japanese era code,
    /// not handling "bce" or "ce"
    ///
    /// Returns (era_start, era_end)
    fn japanese_era_range_for(
        &self,
        era: TinyStr16,
    ) -> Result<(EraStartDate, Option<EraStartDate>), CalendarError> {
        // Avoid linear search by trying well known eras
        if era == tinystr!(16, "reiwa") {
            // Check if we're the last
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
        // Try to avoid linear search by binary searching for the year suffix
        if let Some(year) = era.split('-').nth(1) {
            if let Ok(ref int) = year.parse::<i32>() {
                if let Ok(index) = data.binary_search_by(|(d, _)| d.year.cmp(int)) {
                    #[allow(clippy::expect_used)] // see expect message
                    let (era_start, code) = data
                        .get(index)
                        .expect("Indexing from successful binary search must succeed");
                    // There is a slight chance we hit the case where there are two eras in the same year
                    // There are a couple of rare cases of this, but it's not worth writing a range-based binary search
                    // to catch them since this is an optimization
                    if code == era {
                        return Ok((era_start, data.get(index + 1).map(|e| e.0)));
                    }
                }
            }
        }

        // Avoidance didn't work. Let's find the era manually, searching back from the present
        if let Some((index, (start, _))) = data.iter().enumerate().rev().find(|d| d.1 .1 == era) {
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

    fn single_test_roundtrip(calendar: Ref<Japanese>, era: &str, year: i32, month: u8, day: u8) {
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

    // test that the Gregorian eras roundtrip to Japanese ones
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
        // Heisei did not start until later in the year
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
            CalendarError::UnknownEra("hakuho-672".parse().unwrap(), "Japanese (Modern eras only)"),
        );

        // handle bce/ce
        single_test_roundtrip(calendar, "bce", 100, 3, 1);
        single_test_roundtrip(calendar, "bce", 1, 3, 1);
        single_test_roundtrip(calendar, "ce", 1, 3, 1);
        single_test_roundtrip(calendar, "ce", 100, 3, 1);
        single_test_roundtrip_ext(calendar_ext, "ce", 100, 3, 1);
        single_test_roundtrip(calendar, "ce", 1000, 3, 1);
        single_test_error(calendar, "ce", 0, 3, 1, CalendarError::OutOfRange);
        single_test_error(calendar, "bce", -1, 3, 1, CalendarError::OutOfRange);

        // handle the cases where bce/ce get adjusted to different eras
        // single_test_gregorian_roundtrip(calendar, "ce", 2021, 3, 1, "reiwa", 3);
        single_test_gregorian_roundtrip_ext(calendar_ext, "ce", 1000, 3, 1, "choho-999", 2);
        single_test_gregorian_roundtrip_ext(calendar_ext, "ce", 749, 5, 10, "tenpyokampo-749", 1);
        single_test_gregorian_roundtrip_ext(calendar_ext, "bce", 10, 3, 1, "bce", 10);

        // There were multiple eras in this year
        // This one is from Apr 14 to July 2
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
pub mod julian {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    //! This module contains types and implementations for the Julian calendar.
    //!
    //! ```rust
    //! use icu::calendar::{julian::Julian, Date, DateTime};
    //!
    //! // `Date` type
    //! let date_iso = Date::try_new_iso_date(1970, 1, 2)
    //!     .expect("Failed to initialize ISO Date instance.");
    //! let date_julian = Date::new_from_iso(date_iso, Julian);
    //!
    //! // `DateTime` type
    //! let datetime_iso = DateTime::try_new_iso_datetime(1970, 1, 2, 13, 1, 0)
    //!     .expect("Failed to initialize ISO DateTime instance.");
    //! let datetime_julian = DateTime::new_from_iso(datetime_iso, Julian);
    //!
    //! // `Date` checks
    //! assert_eq!(date_julian.year().number, 1969);
    //! assert_eq!(date_julian.month().ordinal, 12);
    //! assert_eq!(date_julian.day_of_month().0, 20);
    //!
    //! // `DateTime` type
    //! assert_eq!(datetime_julian.date.year().number, 1969);
    //! assert_eq!(datetime_julian.date.month().ordinal, 12);
    //! assert_eq!(datetime_julian.date.day_of_month().0, 20);
    //! assert_eq!(datetime_julian.time.hour.number(), 13);
    //! assert_eq!(datetime_julian.time.minute.number(), 1);
    //! assert_eq!(datetime_julian.time.second.number(), 0);
    //! ```
    
    use crate::any_calendar::AnyCalendarKind;
    use crate::calendar_arithmetic::{ArithmeticDate, CalendarArithmetic};
    use crate::helpers::quotient;
    use crate::iso::Iso;
    use crate::{types, Calendar, CalendarError, Date, DateDuration, DateDurationUnit, DateTime};
    use core::marker::PhantomData;
    use tinystr::tinystr;
    
    // Julian epoch is equivalent to fixed_from_iso of December 30th of 0 year
    // 1st Jan of 1st year Julian is equivalent to December 30th of 0th year of ISO year
    const JULIAN_EPOCH: i32 = -1;
    
    /// The [Julian Calendar]
    ///
    /// The [Julian calendar] is a solar calendar that was used commonly historically, with twelve months.
    ///
    /// This type can be used with [`Date`] or [`DateTime`] to represent dates in this calendar.
    ///
    /// [Julian calendar]: https://en.wikipedia.org/wiki/Julian_calendar
    ///
    /// # Era codes
    ///
    /// This calendar supports two era codes: `"bc"`, and `"ad"`, corresponding to the BC and AD eras
    #[derive(Copy, Clone, Debug, Hash, Default, Eq, PartialEq)]
    #[cfg_attr(test, derive(bolero::generator::TypeGenerator))]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Julian;
    
    /// The inner date type used for representing [`Date`]s of [`Julian`]. See [`Date`] and [`Julian`] for more details.
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    // The inner date type used for representing Date<Julian>
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
    
        /// The calendar-specific year represented by `date`
        /// Julian has the same era scheme as Gregorian
        fn year(&self, date: &Self::DateInner) -> types::FormattableYear {
            crate::gregorian::year_as_gregorian(date.0.year)
        }
    
        /// The calendar-specific month represented by `date`
        fn month(&self, date: &Self::DateInner) -> types::FormattableMonth {
            date.0.solar_month()
        }
    
        /// The calendar-specific day-of-month represented by `date`
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
        /// Construct a new Julian Calendar
        pub fn new() -> Self {
            Self
        }
    
        #[inline(always)]
        const fn is_leap_year_const(year: i32) -> bool {
            year % 4 == 0
        }
    
        // "Fixed" is a day count representation of calendars staring from Jan 1st of year 1 of the Georgian Calendar.
        // The fixed date algorithms are from
        // Dershowitz, Nachum, and Edward M. Reingold. _Calendrical calculations_. Cambridge University Press, 2008.
        //
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1689-L1709
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
    
        /// Convenience function so we can call days_in_year without
        /// needing to construct a full ArithmeticDate
        fn days_in_year_direct(year: i32) -> u32 {
            if Julian::is_leap_year(year) {
                366
            } else {
                365
            }
        }
    
        // Lisp code reference: https://github.com/EdReingold/calendar-code2/blob/1ee51ecfaae6f856b0d7de3e36e9042100b4f424/calendar.l#L1711-L1738
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
        /// Construct new Julian Date.
        ///
        /// Zero and negative years are in BC, with year 0 = 1 BC
        ///
        /// ```rust
        /// use icu::calendar::Date;
        ///
        /// let date_julian = Date::try_new_julian_date(1969, 12, 20)
        ///     .expect("Failed to initialize Julian Date instance.");
        ///
        /// assert_eq!(date_julian.year().number, 1969);
        /// assert_eq!(date_julian.month().ordinal, 12);
        /// assert_eq!(date_julian.day_of_month().0, 20);
        /// ```
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
        /// Construct a new Julian datetime from integers.
        ///
        /// Zero and negative years are in BC, with year 0 = 1 BC
        ///
        /// ```rust
        /// use icu::calendar::DateTime;
        ///
        /// let datetime_julian =
        ///     DateTime::try_new_julian_datetime(1969, 12, 20, 13, 1, 0)
        ///         .expect("Failed to initialize Julian DateTime instance.");
        ///
        /// assert_eq!(datetime_julian.date.year().number, 1969);
        /// assert_eq!(datetime_julian.date.month().ordinal, 12);
        /// assert_eq!(datetime_julian.date.day_of_month().0, 20);
        /// assert_eq!(datetime_julian.time.hour.number(), 13);
        /// assert_eq!(datetime_julian.time.minute.number(), 1);
        /// assert_eq!(datetime_julian.time.second.number(), 0);
        /// ```
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
            // March 1st 200 is same on both calendars
            let iso_date = Date::try_new_iso_date(200, 3, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 200);
            assert_eq!(julian_date.0.month, 3);
            assert_eq!(julian_date.0.day, 1);
    
            // Feb 28th, 200 (iso) = Feb 29th, 200 (julian)
            let iso_date = Date::try_new_iso_date(200, 2, 28).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 200);
            assert_eq!(julian_date.0.month, 2);
            assert_eq!(julian_date.0.day, 29);
    
            // March 1st 400 (iso) = Feb 29th, 400 (julian)
            let iso_date = Date::try_new_iso_date(400, 3, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 400);
            assert_eq!(julian_date.0.month, 2);
            assert_eq!(julian_date.0.day, 29);
    
            // Jan 1st, 2022 (iso) = Dec 19, 2021 (julian)
            let iso_date = Date::try_new_iso_date(2022, 1, 1).unwrap();
            let julian_date = Julian.date_from_iso(iso_date);
            assert_eq!(julian_date.0.year, 2021);
            assert_eq!(julian_date.0.month, 12);
            assert_eq!(julian_date.0.day, 19);
        }
    
        #[test]
        fn test_day_julian_to_iso() {
            // March 1st 200 is same on both calendars
            let julian_date = Date::try_new_julian_date(200, 3, 1).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(200, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);
    
            // Feb 28th, 200 (iso) = Feb 29th, 200 (julian)
            let julian_date = Date::try_new_julian_date(200, 2, 29).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(200, 2, 28).unwrap();
            assert_eq!(iso_date, iso_expected_date);
    
            // March 1st 400 (iso) = Feb 29th, 400 (julian)
            let julian_date = Date::try_new_julian_date(400, 2, 29).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(400, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);
    
            // Jan 1st, 2022 (iso) = Dec 19, 2021 (julian)
            let julian_date = Date::try_new_julian_date(2021, 12, 19).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(2022, 1, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);
    
            // March 1st, 2022 (iso) = Feb 16, 2022 (julian)
            let julian_date = Date::try_new_julian_date(2022, 2, 16).unwrap();
            let iso_date = Julian.date_to_iso(julian_date.inner());
            let iso_expected_date = Date::try_new_iso_date(2022, 3, 1).unwrap();
            assert_eq!(iso_date, iso_expected_date);
        }
    
        #[test]
        fn test_roundtrip_negative() {
            // https://github.com/unicode-org/icu4x/issues/2254
            let iso_date = Date::try_new_iso_date(-1000, 3, 3).unwrap();
            let julian = iso_date.to_calendar(Julian::new());
            let recovered_iso = julian.to_iso();
            assert_eq!(iso_date, recovered_iso);
        }
    }
    }
pub mod provider {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    //! 🚧 \[Unstable\] Data provider struct definitions for this ICU4X component.
    //!
    //! <div class="stab unstable">
    //! 🚧 This code is considered unstable; it may change at any time, in breaking or non-breaking ways,
    //! including in SemVer minor releases. While the serde representation of data structs is guaranteed
    //! to be stable, their Rust representation might not be. Use with caution.
    //! </div>
    //!
    //! Read more about data providers: [`icu_provider`]
    
    // Provider structs must be stable
    #![allow(clippy::exhaustive_structs, clippy::exhaustive_enums)]
    
    use crate::types::IsoWeekday;
    use core::str::FromStr;
    use icu_provider::{yoke, zerofrom};
    use tinystr::TinyStr16;
    use zerovec::ZeroVec;
    
    /// The date at which an era started
    ///
    /// The order of fields in this struct is important!
    ///
    /// <div class="stab unstable">
    /// 🚧 This code is considered unstable; it may change at any time, in breaking or non-breaking ways,
    /// including in SemVer minor releases. While the serde representation of data structs is guaranteed
    /// to be stable, their Rust representation might not be. Use with caution.
    /// </div>
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
        /// The year the era started in
        pub year: i32,
        /// The month the era started in
        pub month: u8,
        /// The day the era started in
        pub day: u8,
    }
    
    /// A data structure containing the necessary era data for constructing a
    /// [`Japanese`](crate::japanese::Japanese) calendar object
    ///
    /// <div class="stab unstable">
    /// 🚧 This code is considered unstable; it may change at any time, in breaking or non-breaking ways,
    /// including in SemVer minor releases. While the serde representation of data structs is guaranteed
    /// to be stable, their Rust representation might not be. Use with caution.
    /// </div>
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
        /// A map from era start dates to their era codes
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
    
    /// An ICU4X mapping to a subset of CLDR weekData.
    /// See CLDR-JSON's weekData.json for more context.
    ///
    /// <div class="stab unstable">
    /// 🚧 This code is considered unstable; it may change at any time, in breaking or non-breaking ways,
    /// including in SemVer minor releases. While the serde representation of data structs is guaranteed
    /// to be stable, their Rust representation might not be. Use with caution.
    /// </div>
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
        /// The first day of a week.
        pub first_weekday: IsoWeekday,
        /// For a given week, the minimum number of that week's days present in a given month or year for the week to be considered part of that month or year.
        pub min_week_days: u8,
    }
    }
pub mod types {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    //! This module contains various types used by `icu_calendar` and `icu_datetime`
    
    use crate::error::CalendarError;
    use crate::helpers;
    use core::convert::TryFrom;
    use core::convert::TryInto;
    use core::fmt;
    use core::str::FromStr;
    use tinystr::{TinyStr16, TinyStr4};
    use zerovec::maps::ZeroMapKV;
    use zerovec::ule::AsULE;
    
    /// The era of a particular date
    ///
    /// Different calendars use different era codes, see their documentation
    /// for details.
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
    
    /// Representation of a formattable year.
    ///
    /// More fields may be added in the future, for things like
    /// the cyclic or extended year
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[non_exhaustive]
    pub struct FormattableYear {
        /// The era containing the year.
        pub era: Era,
    
        /// The year number in the current era (usually 1-based).
        pub number: i32,
    
        /// The related ISO year. This is normally the ISO (proleptic Gregorian) year having the greatest
        /// overlap with the calendar year. It is used in certain date formatting patterns.
        ///
        /// Can be None if the calendar does not typically use related_iso (and CLDR does not contain patterns
        /// using it)
        pub related_iso: Option<i32>,
    }
    
    impl FormattableYear {
        /// Construct a new Year given an era and number
        ///
        /// Other fields can be set mutably after construction
        /// as needed
        pub fn new(era: Era, number: i32) -> Self {
            Self {
                era,
                number,
                related_iso: None,
            }
        }
    }
    
    /// Representation of a month in a year
    ///
    /// Month codes typically look like `M01`, `M02`, etc, but can handle leap months
    /// (`M03L`) in lunar calendars. Solar calendars will have codes between `M01` and `M12`
    /// potentially with an `M13` for epagomenal months. Check the docs for a particular calendar
    /// for details on what its month codes are.
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
    
    /// Representation of a formattable month.
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct FormattableMonth {
        /// The month number in this given year. For calendars with leap months, all months after
        /// the leap month will end up with an incremented number.
        ///
        /// In general, prefer using the month code in generic code.
        pub ordinal: u32,
    
        /// The month code, used to distinguish months during leap years.
        pub code: MonthCode,
    }
    
    /// A struct containing various details about the position of the day within a year. It is returned
    // by the [`day_of_year_info()`](trait.DateInput.html#tymethod.day_of_year_info) method of the
    // [`DateInput`] trait.
    #[derive(Copy, Clone, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct DayOfYearInfo {
        /// The current day of the year, 1-based.
        pub day_of_year: u32,
        /// The number of days in a year.
        pub days_in_year: u32,
        /// The previous year.
        pub prev_year: FormattableYear,
        /// The number of days in the previous year.
        pub days_in_prev_year: u32,
        /// The next year.
        pub next_year: FormattableYear,
    }
    
    /// A day number in a month. Usually 1-based.
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct DayOfMonth(pub u32);
    
    /// A week number in a month. Usually 1-based.
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct WeekOfMonth(pub u32);
    
    /// A week number in a year. Usually 1-based.
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this is a newtype
    pub struct WeekOfYear(pub u32);
    
    /// A day of week in month. 1-based.
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
    
    /// This macro defines a struct for 0-based date fields: hours, minutes, seconds
    /// and fractional seconds. Each unit is bounded by a range. The traits implemented
    /// here will return a Result on whether or not the unit is in range from the given
    /// input.
    macro_rules! dt_unit {
        ($name:ident, $storage:ident, $value:expr, $docs:expr) => {
            #[doc=$docs]
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
            pub struct $name($storage);
    
            impl $name {
                /// Gets the numeric value for this component.
                pub const fn number(self) -> $storage {
                    self.0
                }
    
                /// Creates a new value at 0.
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
                /// Attempts to add two values.
                /// Returns `Some` if the sum is within bounds.
                /// Returns `None` if the sum is out of bounds.
                pub fn try_add(self, other: $storage) -> Option<Self> {
                    let sum = self.0.saturating_add(other);
                    if sum > $value {
                        None
                    } else {
                        Some(Self(sum))
                    }
                }
    
                /// Attempts to subtract two values.
                /// Returns `Some` if the difference is within bounds.
                /// Returns `None` if the difference is out of bounds.
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
    
        // middle of bounds
        assert_eq!(
            hour.try_add(HOUR_VALUE - 1),
            Some(IsoHour(HOUR_VALUE + (HOUR_VALUE - 1)))
        );
        assert_eq!(
            hour.try_sub(HOUR_VALUE - 1),
            Some(IsoHour(HOUR_VALUE - (HOUR_VALUE - 1)))
        );
    
        // edge of bounds
        assert_eq!(hour.try_add(HOUR_MAX - HOUR_VALUE), Some(IsoHour(HOUR_MAX)));
        assert_eq!(hour.try_sub(HOUR_VALUE), Some(IsoHour(0)));
    
        // out of bounds
        assert_eq!(hour.try_add(1 + HOUR_MAX - HOUR_VALUE), None);
        assert_eq!(hour.try_sub(1 + HOUR_VALUE), None);
    }
    
    #[test]
    fn test_iso_minute_arithmetic() {
        const MINUTE_MAX: u8 = 60;
        const MINUTE_VALUE: u8 = 5;
        let minute = IsoMinute(MINUTE_VALUE);
    
        // middle of bounds
        assert_eq!(
            minute.try_add(MINUTE_VALUE - 1),
            Some(IsoMinute(MINUTE_VALUE + (MINUTE_VALUE - 1)))
        );
        assert_eq!(
            minute.try_sub(MINUTE_VALUE - 1),
            Some(IsoMinute(MINUTE_VALUE - (MINUTE_VALUE - 1)))
        );
    
        // edge of bounds
        assert_eq!(
            minute.try_add(MINUTE_MAX - MINUTE_VALUE),
            Some(IsoMinute(MINUTE_MAX))
        );
        assert_eq!(minute.try_sub(MINUTE_VALUE), Some(IsoMinute(0)));
    
        // out of bounds
        assert_eq!(minute.try_add(1 + MINUTE_MAX - MINUTE_VALUE), None);
        assert_eq!(minute.try_sub(1 + MINUTE_VALUE), None);
    }
    
    #[test]
    fn test_iso_second_arithmetic() {
        const SECOND_MAX: u8 = 61;
        const SECOND_VALUE: u8 = 5;
        let second = IsoSecond(SECOND_VALUE);
    
        // middle of bounds
        assert_eq!(
            second.try_add(SECOND_VALUE - 1),
            Some(IsoSecond(SECOND_VALUE + (SECOND_VALUE - 1)))
        );
        assert_eq!(
            second.try_sub(SECOND_VALUE - 1),
            Some(IsoSecond(SECOND_VALUE - (SECOND_VALUE - 1)))
        );
    
        // edge of bounds
        assert_eq!(
            second.try_add(SECOND_MAX - SECOND_VALUE),
            Some(IsoSecond(SECOND_MAX))
        );
        assert_eq!(second.try_sub(SECOND_VALUE), Some(IsoSecond(0)));
    
        // out of bounds
        assert_eq!(second.try_add(1 + SECOND_MAX - SECOND_VALUE), None);
        assert_eq!(second.try_sub(1 + SECOND_VALUE), None);
    }
    
    #[test]
    fn test_iso_nano_second_arithmetic() {
        const NANO_SECOND_MAX: u32 = 999_999_999;
        const NANO_SECOND_VALUE: u32 = 5;
        let nano_second = NanoSecond(NANO_SECOND_VALUE);
    
        // middle of bounds
        assert_eq!(
            nano_second.try_add(NANO_SECOND_VALUE - 1),
            Some(NanoSecond(NANO_SECOND_VALUE + (NANO_SECOND_VALUE - 1)))
        );
        assert_eq!(
            nano_second.try_sub(NANO_SECOND_VALUE - 1),
            Some(NanoSecond(NANO_SECOND_VALUE - (NANO_SECOND_VALUE - 1)))
        );
    
        // edge of bounds
        assert_eq!(
            nano_second.try_add(NANO_SECOND_MAX - NANO_SECOND_VALUE),
            Some(NanoSecond(NANO_SECOND_MAX))
        );
        assert_eq!(nano_second.try_sub(NANO_SECOND_VALUE), Some(NanoSecond(0)));
    
        // out of bounds
        assert_eq!(
            nano_second.try_add(1 + NANO_SECOND_MAX - NANO_SECOND_VALUE),
            None
        );
        assert_eq!(nano_second.try_sub(1 + NANO_SECOND_VALUE), None);
    }
    
    /// A representation of a time in hours, minutes, seconds, and nanoseconds
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct Time {
        /// 0-based hour.
        pub hour: IsoHour,
    
        /// 0-based minute.
        pub minute: IsoMinute,
    
        /// 0-based second.
        pub second: IsoSecond,
    
        /// Fractional second
        pub nanosecond: NanoSecond,
    }
    
    impl Time {
        /// Construct a new [`Time`], without validating that all components are in range
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
    
        /// Construct a new [`Time`], whilst validating that all components are in range
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
    
        /// Takes a number of minutes, which could be positive or negative, and returns the Time
        /// and the day number, which could be positive or negative.
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
    
    /// A weekday in a 7-day week, according to ISO-8601.
    ///
    /// The discriminant values correspond to ISO-8601 weekday numbers (Monday = 1, Sunday = 7).
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::calendar::types::IsoWeekday;
    ///
    /// assert_eq!(1, IsoWeekday::Monday as usize);
    /// assert_eq!(7, IsoWeekday::Sunday as usize);
    /// ```
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
        /// Convert from an ISO-8601 weekday number to an [`IsoWeekday`] enum. 0 is automatically converted
        /// to 7 (Sunday). If the number is out of range, it is interpreted modulo 7.
        ///
        /// # Examples
        ///
        /// ```
        /// use icu::calendar::types::IsoWeekday;
        ///
        /// assert_eq!(IsoWeekday::Sunday, IsoWeekday::from(0));
        /// assert_eq!(IsoWeekday::Monday, IsoWeekday::from(1));
        /// assert_eq!(IsoWeekday::Sunday, IsoWeekday::from(7));
        /// assert_eq!(IsoWeekday::Monday, IsoWeekday::from(8));
        /// ```
        fn from(input: usize) -> Self {
            let mut ordinal = (input % 7) as i8;
            if ordinal == 0 {
                ordinal = 7;
            }
            unsafe { core::mem::transmute(ordinal) }
        }
    }
    }
mod week_of {// This file is part of ICU4X. For terms of use, please see the file
    // called LICENSE at the top level of the ICU4X source tree
    // (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).
    
    use crate::{
        error::CalendarError,
        provider::WeekDataV1,
        types::{DayOfMonth, DayOfYearInfo, IsoWeekday, WeekOfMonth},
    };
    use icu_provider::prelude::*;
    
    /// Minimum number of days in a month unit required for using this module
    pub const MIN_UNIT_DAYS: u16 = 14;
    
    /// Calculator for week-of-month and week-of-year based on locale-specific configurations.
    #[derive(Clone, Copy, Debug)]
    #[non_exhaustive]
    pub struct WeekCalculator {
        /// The first day of a week.
        pub first_weekday: IsoWeekday,
        /// For a given week, the minimum number of that week's days present in a given month or year
        /// for the week to be considered part of that month or year.
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
        /// Creates a new [`WeekCalculator`] from locale data.
        ///
        /// [📚 Help choosing a constructor](icu_provider::constructors)
        /// <div class="stab unstable">
        /// ⚠️ The bounds on this function may change over time, including in SemVer minor releases.
        /// </div>
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
    
        /// Returns the week of month according to a calendar with min_week_days = 1.
        ///
        /// This is different from what the UTS35 spec describes [1] but the latter is
        /// missing a month of week-of-month field so following the spec would result
        /// in inconsistencies (e.g. in the ISO calendar 2021-01-01 is the last week
        /// of December but 'MMMMW' would have it formatted as 'week 5 of January').
        ///
        /// # Examples
        ///
        /// ```
        /// use icu_calendar::types::{DayOfMonth, IsoWeekday, WeekOfMonth};
        /// use icu_calendar::week::WeekCalculator;
        ///
        /// let week_calculator = WeekCalculator::try_new_unstable(
        ///     &icu_testdata::unstable(),
        ///     &icu_locid::locale!("en-GB").into(),
        /// )
        /// .expect("Data exists");
        ///
        /// // Wednesday the 10th is in week 2:
        /// assert_eq!(
        ///     WeekOfMonth(2),
        ///     week_calculator.week_of_month(DayOfMonth(10), IsoWeekday::Wednesday)
        /// );
        /// ```
        ///
        /// [1]: https://www.unicode.org/reports/tr35/tr35-55/tr35-dates.html#Date_Patterns_Week_Of_Year
        pub fn week_of_month(&self, day_of_month: DayOfMonth, iso_weekday: IsoWeekday) -> WeekOfMonth {
            WeekOfMonth(simple_week_of(self.first_weekday, day_of_month.0 as u16, iso_weekday) as u32)
        }
    
        /// Returns the week of year according to the weekday and [`DayOfYearInfo`].
        ///
        /// # Examples
        ///
        /// ```
        /// use icu_calendar::types::{DayOfMonth, IsoWeekday};
        /// use icu_calendar::week::{RelativeUnit, WeekCalculator, WeekOf};
        /// use icu_calendar::Date;
        ///
        /// let week_calculator = WeekCalculator::try_new_unstable(
        ///     &icu_testdata::unstable(),
        ///     &icu_locid::locale!("en-GB").into(),
        /// )
        /// .expect("Data exists");
        ///
        /// let iso_date = Date::try_new_iso_date(2022, 8, 26).unwrap();
        ///
        /// // Friday August 26 is in week 34 of year 2022:
        /// assert_eq!(
        ///     WeekOf {
        ///         unit: RelativeUnit::Current,
        ///         week: 34
        ///     },
        ///     week_calculator
        ///         .week_of_year(iso_date.day_of_year_info(), IsoWeekday::Friday)
        ///         .unwrap()
        /// );
        /// ```
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
    
        /// Returns the zero based index of `weekday` vs this calendar's start of week.
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
    
    /// Returns the weekday that's `num_days` after `weekday`.
    fn add_to_weekday(weekday: IsoWeekday, num_days: i32) -> IsoWeekday {
        let new_weekday = (7 + (weekday as i32) + (num_days % 7)) % 7;
        IsoWeekday::from(new_weekday as usize)
    }
    
    /// Which year or month that a calendar assigns a week to relative to the year/month
    /// the week is in.
    #[derive(Clone, Copy, Debug, PartialEq)]
    #[allow(clippy::enum_variant_names)]
    enum RelativeWeek {
        /// A week that is assigned to the last week of the previous year/month. e.g. 2021-01-01 is week 54 of 2020 per the ISO calendar.
        LastWeekOfPreviousUnit,
        /// A week that's assigned to the current year/month. The offset is 1-based. e.g. 2021-01-11 is week 2 of 2021 per the ISO calendar so would be WeekOfCurrentUnit(2).
        WeekOfCurrentUnit(u16),
        /// A week that is assigned to the first week of the next year/month. e.g. 2019-12-31 is week 1 of 2020 per the ISO calendar.
        FirstWeekOfNextUnit,
    }
    
    /// Information about a year or month.
    struct UnitInfo {
        /// The weekday of this year/month's first day.
        first_day: IsoWeekday,
        /// The number of days in this year/month.
        duration_days: u16,
    }
    
    impl UnitInfo {
        /// Creates a UnitInfo for a given year or month.
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
    
        /// Returns the start of this unit's first week.
        ///
        /// The returned value can be negative if this unit's first week started during the previous
        /// unit.
        fn first_week_offset(&self, calendar: &WeekCalculator) -> i8 {
            let first_day_index = calendar.weekday_index(self.first_day);
            if 7 - first_day_index >= calendar.min_week_days as i8 {
                -first_day_index
            } else {
                7 - first_day_index
            }
        }
    
        /// Returns the number of weeks in this unit according to `calendar`.
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
    
        /// Returns the week number for the given day in this unit.
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
    
    /// The year or month that a calendar assigns a week to relative to the year/month that it is in.
    #[derive(Debug, PartialEq)]
    #[allow(clippy::exhaustive_enums)] // this type is stable
    pub enum RelativeUnit {
        /// A week that is assigned to previous year/month. e.g. 2021-01-01 is week 54 of 2020 per the ISO calendar.
        Previous,
        /// A week that's assigned to the current year/month. e.g. 2021-01-11 is week 2 of 2021 per the ISO calendar.
        Current,
        /// A week that is assigned to the next year/month. e.g. 2019-12-31 is week 1 of 2020 per the ISO calendar.
        Next,
    }
    
    /// The week number assigned to a given week according to a calendar.
    #[derive(Debug, PartialEq)]
    #[allow(clippy::exhaustive_structs)] // this type is stable
    pub struct WeekOf {
        /// Week of month/year. 1 based.
        pub week: u16,
        /// The month/year that this week is in, relative to the month/year of the input date.
        pub unit: RelativeUnit,
    }
    
    /// Computes & returns the week of given month/year according to `calendar`.
    ///
    /// # Arguments
    ///  - calendar: Calendar information used to compute the week number.
    ///  - num_days_in_previous_unit: The number of days in the preceding month/year.
    ///  - num_days_in_unit: The number of days in the month/year.
    ///  - day: 1-based day of month/year.
    ///  - week_day: The weekday of `day`..
    pub fn week_of(
        calendar: &WeekCalculator,
        num_days_in_previous_unit: u16,
        num_days_in_unit: u16,
        day: u16,
        week_day: IsoWeekday,
    ) -> Result<WeekOf, CalendarError> {
        let current = UnitInfo::new(
            // The first day of this month/year is (day - 1) days from `day`.
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
    
    /// Computes & returns the week of given month or year according to a calendar with min_week_days = 1.
    ///
    /// Does not know anything about the unit size (month or year), and will just assume the date falls
    /// within whatever unit that is being considered. In other words, this function returns strictly increasing
    /// values as `day` increases, unlike [`week_of()`] which is cyclic.
    ///
    /// # Arguments
    ///  - first_weekday: The first day of a week.
    ///  - day: 1-based day of the month or year.
    ///  - week_day: The weekday of `day`.
    pub fn simple_week_of(first_weekday: IsoWeekday, day: u16, week_day: IsoWeekday) -> u16 {
        let calendar = WeekCalculator {
            first_weekday,
            min_week_days: 1,
        };
    
        #[allow(clippy::unwrap_used)] // week_of should can't fail with MIN_UNIT_DAYS
        week_of(
            &calendar,
            // The duration of the previous unit does not influence the result if min_week_days = 1
            // so we only need to use a valid value.
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
            // 4 days in first & last week.
            assert_eq!(
                UnitInfo::new(IsoWeekday::Thursday, 4 + 2 * 7 + 4)?.num_weeks(&ISO_CALENDAR),
                4
            );
            // 3 days in first week, 4 in last week.
            assert_eq!(
                UnitInfo::new(IsoWeekday::Friday, 3 + 2 * 7 + 4)?.num_weeks(&ISO_CALENDAR),
                3
            );
            // 3 days in first & last week.
            assert_eq!(
                UnitInfo::new(IsoWeekday::Friday, 3 + 2 * 7 + 3)?.num_weeks(&ISO_CALENDAR),
                2
            );
    
            // 1 day in first & last week.
            assert_eq!(
                UnitInfo::new(IsoWeekday::Saturday, 1 + 2 * 7 + 1)?.num_weeks(&US_CALENDAR),
                4
            );
            Ok(())
        }
    
        /// Uses enumeration & bucketing to assign each day of a month or year `unit` to a week.
        ///
        /// This alternative implementation serves as an exhaustive safety check
        /// of relative_week() (in addition to the manual test points used
        /// for testing week_of()).
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
                            let unit = UnitInfo::new(IsoWeekday::from(start_of_unit), unit_duration)?;
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
    
            // First day of year is a Thursday.
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20180101)?,
                WeekOf {
                    week: 1,
                    unit: RelativeUnit::Current,
                }
            );
            // First day of year is a Friday.
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20210101)?,
                WeekOf {
                    week: 5,
                    unit: RelativeUnit::Previous,
                }
            );
    
            // The month ends on a Wednesday.
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20200930)?,
                WeekOf {
                    week: 1,
                    unit: RelativeUnit::Next,
                }
            );
            // The month ends on a Thursday.
            assert_eq!(
                week_of_month_from_iso_date(&ISO_CALENDAR, 20201231)?,
                WeekOf {
                    week: 5,
                    unit: RelativeUnit::Current,
                }
            );
    
            // US calendar always assigns the week to the current month. 2020-12-31 is a Thursday.
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
        // The 1st is a Monday and the week starts on Mondays.
        assert_eq!(
            simple_week_of(IsoWeekday::Monday, 2, IsoWeekday::Tuesday),
            1
        );
        assert_eq!(simple_week_of(IsoWeekday::Monday, 7, IsoWeekday::Sunday), 1);
        assert_eq!(simple_week_of(IsoWeekday::Monday, 8, IsoWeekday::Monday), 2);
    
        // The 1st is a Wednesday and the week starts on Tuesdays.
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
    
        // The 1st is a Monday and the week starts on Sundays.
        assert_eq!(
            simple_week_of(IsoWeekday::Sunday, 26, IsoWeekday::Friday),
            4
        );
    }
    }

pub mod week {
    //! Functions for week-of-month and week-of-year arithmetic.
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

/// Re-export of [`CalendarError`].
#[doc(no_inline)]
pub use CalendarError as Error;
