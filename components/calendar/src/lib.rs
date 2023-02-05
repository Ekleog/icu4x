
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
mod error;
pub mod ethiopian;
mod fuzz;
pub mod gregorian;
mod helpers;
pub mod indian;
pub mod iso;
pub mod japanese;
pub mod julian;
pub mod provider;
pub mod types;
mod week_of;

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
