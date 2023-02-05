
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

pub mod any_calendar;
pub mod buddhist;
mod calendar;
mod calendar_arithmetic;
pub mod coptic;
mod duration;
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
