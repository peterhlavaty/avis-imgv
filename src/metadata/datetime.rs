//! The timestamp format EXIF uses, and arithmetic on it.
//!
//! `YYYY:MM:DD HH:MM:SS`, nineteen ASCII characters, no time zone. The lack of
//! a zone is the point: shifting a badly set camera clock is arithmetic on
//! wall clock time, and nothing about it depends on where the photograph was
//! taken.
//!
//! The conversion to and from a day number is the standard proleptic Gregorian
//! one, exact for every year this will ever see.

use std::fmt;

/// Characters an EXIF timestamp occupies. Cameras write a trailing NUL, which
/// is not counted here and is left alone when one is rewritten.
pub const EXIF_LEN: usize = 19;

/// A wall clock date and time, to the second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

impl Timestamp {
    /// Reads `YYYY:MM:DD HH:MM:SS`.
    ///
    /// Also accepts the ISO spelling with dashes and a `T`, because that is
    /// what XMP and some cameras write, and a trailing NUL or spaces.
    pub fn parse(text: &str) -> Option<Timestamp> {
        let text = text.trim_matches(|c: char| c == '\0' || c.is_whitespace());
        let bytes = text.as_bytes();

        if bytes.len() < EXIF_LEN {
            return None;
        }

        let number = |from: usize, to: usize| text.get(from..to)?.parse::<u32>().ok();

        let timestamp = Timestamp {
            year: number(0, 4)? as i32,
            month: number(5, 7)?,
            day: number(8, 10)?,
            hour: number(11, 13)?,
            minute: number(14, 16)?,
            second: number(17, 19)?,
        };

        timestamp.is_valid().then_some(timestamp)
    }

    /// Whether this is a date that exists and a time that happens.
    ///
    /// Cameras with a flat battery write `0000:00:00 00:00:00`, which is not a
    /// timestamp and must not be shifted into one.
    fn is_valid(&self) -> bool {
        self.month >= 1
            && self.month <= 12
            && self.day >= 1
            && self.day <= days_in_month(self.year, self.month)
            && self.hour < 24
            && self.minute < 60
            // Leap seconds are written by nobody, but rejecting them would be
            // worse than accepting them.
            && self.second < 61
    }

    /// Seconds since the start of 1970, which is only ever used as a number to
    /// do arithmetic on.
    pub fn to_seconds(self) -> i64 {
        let days = days_from_civil(self.year, self.month, self.day);

        days * 86_400
            + i64::from(self.hour) * 3600
            + i64::from(self.minute) * 60
            + i64::from(self.second)
    }

    /// The timestamp `seconds` after the start of 1970.
    pub fn from_seconds(seconds: i64) -> Timestamp {
        // Rust truncates division towards zero, so a negative remainder has to
        // be carried back into the day rather than left as a negative hour.
        let mut days = seconds.div_euclid(86_400);
        let mut rest = seconds.rem_euclid(86_400);

        if rest < 0 {
            rest += 86_400;
            days -= 1;
        }

        let (year, month, day) = civil_from_days(days);

        Timestamp {
            year,
            month,
            day,
            hour: (rest / 3600) as u32,
            minute: (rest % 3600 / 60) as u32,
            second: (rest % 60) as u32,
        }
    }

    /// This timestamp moved by `seconds`, forwards or backwards.
    pub fn shifted(self, seconds: i64) -> Timestamp {
        Timestamp::from_seconds(self.to_seconds() + seconds)
    }

    /// Written back the way EXIF wants it, always [`EXIF_LEN`] characters.
    ///
    /// A year outside four digits cannot be written, and is clamped rather
    /// than allowed to produce a string of the wrong length: an EXIF value is
    /// rewritten in place, so its length is not ours to change.
    pub fn to_exif(self) -> String {
        format!(
            "{:04}:{:02}:{:02} {:02}:{:02}:{:02}",
            self.year.clamp(0, 9999),
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        )
    }

    /// The date alone, as it would go in a file name.
    pub fn to_date(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// The time alone, as it would go in a file name.
    pub fn to_time(self) -> String {
        format!("{:02}-{:02}-{:02}", self.hour, self.minute, self.second)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_exif())
    }
}

fn is_leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(year) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Days from 1970-01-01 to `year-month-day`, proleptic Gregorian.
///
/// The calendar is shifted so it starts in March, which puts the leap day at
/// the end of the year and makes the whole thing a handful of divisions.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;

    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

/// The inverse of [`days_from_civil`].
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;

    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);

    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = shifted_month + if shifted_month < 10 { 3 } else { -9 };

    (
        (year + i64::from(month <= 2)) as i32,
        month as u32,
        day as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Timestamp {
        Timestamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    #[test]
    fn reads_what_a_camera_writes() {
        assert_eq!(
            Timestamp::parse("2024:11:06 22:07:19"),
            Some(at(2024, 11, 6, 22, 7, 19))
        );
    }

    #[test]
    fn reads_a_value_with_its_trailing_nul() {
        assert_eq!(
            Timestamp::parse("2024:11:06 22:07:19\0"),
            Some(at(2024, 11, 6, 22, 7, 19))
        );
    }

    #[test]
    fn reads_the_iso_spelling_too() {
        assert_eq!(
            Timestamp::parse("2024-11-06T22:07:19"),
            Some(at(2024, 11, 6, 22, 7, 19))
        );
    }

    #[test]
    fn a_flat_battery_is_not_a_timestamp() {
        assert_eq!(Timestamp::parse("0000:00:00 00:00:00"), None);
    }

    #[test]
    fn nonsense_is_rejected_rather_than_guessed_at() {
        assert_eq!(Timestamp::parse(""), None);
        assert_eq!(Timestamp::parse("yesterday"), None);
        assert_eq!(Timestamp::parse("2024:13:06 22:07:19"), None);
        assert_eq!(Timestamp::parse("2024:11:31 22:07:19"), None);
        assert_eq!(Timestamp::parse("2024:11:06 25:07:19"), None);
    }

    #[test]
    fn february_has_a_twenty_ninth_only_in_a_leap_year() {
        assert!(Timestamp::parse("2024:02:29 12:00:00").is_some());
        assert!(Timestamp::parse("2023:02:29 12:00:00").is_none());
        // A century is not a leap year unless it divides by four hundred.
        assert!(Timestamp::parse("2000:02:29 12:00:00").is_some());
        assert!(Timestamp::parse("1900:02:29 12:00:00").is_none());
    }

    #[test]
    fn writing_gives_back_what_was_read() {
        let text = "2024:11:06 22:07:19";
        assert_eq!(Timestamp::parse(text).unwrap().to_exif(), text);
        assert_eq!(Timestamp::parse(text).unwrap().to_exif().len(), EXIF_LEN);
    }

    #[test]
    fn the_epoch_is_where_it_should_be() {
        assert_eq!(at(1970, 1, 1, 0, 0, 0).to_seconds(), 0);
        assert_eq!(Timestamp::from_seconds(0), at(1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn seconds_round_trip_over_a_long_span() {
        let mut timestamp = at(1901, 3, 1, 0, 0, 0);

        // Every day for two centuries, at an awkward time of day.
        for _ in 0..(200 * 365) {
            timestamp = timestamp.shifted(86_400);
            assert_eq!(
                Timestamp::from_seconds(timestamp.to_seconds()),
                timestamp,
                "{timestamp}"
            );
        }

        assert!(timestamp.year > 2090);
    }

    #[test]
    fn an_hour_forward_crosses_midnight() {
        let midnight = at(2024, 11, 6, 23, 30, 0).shifted(3600);
        assert_eq!(midnight, at(2024, 11, 7, 0, 30, 0));
    }

    #[test]
    fn an_hour_back_crosses_it_the_other_way() {
        let before = at(2024, 11, 7, 0, 30, 0).shifted(-3600);
        assert_eq!(before, at(2024, 11, 6, 23, 30, 0));
    }

    #[test]
    fn a_shift_crosses_a_year_and_a_leap_day() {
        // 2024 is a leap year, so the last day of February is the 29th.
        let shifted = at(2024, 2, 28, 12, 0, 0).shifted(86_400);
        assert_eq!(shifted, at(2024, 2, 29, 12, 0, 0));

        let new_year = at(2023, 12, 31, 23, 0, 0).shifted(3600);
        assert_eq!(new_year, at(2024, 1, 1, 0, 0, 0));
    }

    #[test]
    fn a_shift_before_the_epoch_still_works() {
        let shifted = at(1970, 1, 1, 0, 0, 0).shifted(-1);
        assert_eq!(shifted, at(1969, 12, 31, 23, 59, 59));
    }

    #[test]
    fn shifting_by_nothing_changes_nothing() {
        let timestamp = at(2024, 11, 6, 22, 7, 19);
        assert_eq!(timestamp.shifted(0), timestamp);
    }

    #[test]
    fn shifting_back_and_forth_returns_to_the_start() {
        let timestamp = at(2024, 11, 6, 22, 7, 19);
        let offset = 5 * 86_400 + 3 * 3600 + 17 * 60 + 42;

        assert_eq!(timestamp.shifted(offset).shifted(-offset), timestamp);
    }

    #[test]
    fn the_file_name_forms_have_no_characters_a_path_would_object_to() {
        let timestamp = at(2024, 11, 6, 22, 7, 19);

        assert_eq!(timestamp.to_date(), "2024-11-06");
        assert_eq!(timestamp.to_time(), "22-07-19");
    }

    #[test]
    fn timestamps_order_by_when_they_happened() {
        let mut all = [
            at(2024, 11, 6, 22, 7, 19),
            at(2023, 1, 1, 0, 0, 0),
            at(2024, 11, 6, 6, 0, 0),
        ];
        all.sort();

        assert_eq!(all[0].year, 2023);
        assert_eq!(all[1].hour, 6);
        assert_eq!(all[2].hour, 22);
    }
}
