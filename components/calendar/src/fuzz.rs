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
