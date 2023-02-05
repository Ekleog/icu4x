
#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

// Make sure inherent docs go first
mod date;
mod datetime;

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
