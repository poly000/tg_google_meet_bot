use crate::calendar3;

use calendar3::chrono::{self, Duration, FixedOffset, ParseError, TimeZone};
use chrono::{DateTime, Utc};
use chrono::{NaiveDate, NaiveTime};

/// input: UTC+8 time string
pub fn parse_time_to_utc<Tz: TimeZone>(
    input: &str,
    now: DateTime<Tz>,
) -> Result<DateTime<Utc>, ParseError>
where
    Tz::Offset: Send,
{
    let mut input = input.split_whitespace();
    let time = input.next().map_or(Ok(now.time()), |time| {
        NaiveTime::parse_from_str(time.trim(), "%H:%M")
    })?;
    let date = input.next().map_or_else(
        || {
            Ok(if time >= now.time() {
                // you can manually specify a past time for the meet
                now.date_naive()
            } else {
                now.date_naive().succ_opt().unwrap()
            })
        },
        |date| NaiveDate::parse_from_str(date, "%d/%m/%Y"),
    )?;

    Ok(date
        .and_time(time)
        .and_utc()
        .checked_sub_signed(Duration::hours(8))
        .unwrap())
}

pub fn utc8_now() -> DateTime<FixedOffset> {
    FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc())
}

#[cfg(test)]
mod tests {
    use calendar3::chrono::format::ParseErrorKind;

    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn succ_day_on_previous_time() {
        assert_eq!(
            parse_time_to_utc(
                "12:00",
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|dt| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())),
            Ok(DateTime::parse_from_rfc3339("2023-04-01T12:00:00+08:00").unwrap())
        );
        assert_eq!(
            parse_time_to_utc(
                "8:00",
                DateTime::parse_from_rfc3339("2023-04-01T10:00:00+08:00").unwrap()
            )
            .map(|dt| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())),
            Ok(DateTime::parse_from_rfc3339("2023-04-02T08:00:00+08:00").unwrap())
        );
    }

    #[test]
    fn return_now_on_empty_time() {
        let now = utc8_now();
        assert_eq!(parse_time_to_utc("", now), Ok(now.naive_utc().and_utc()))
    }

    #[test]
    fn test_parse_date() {
        let now = utc8_now();
        assert_eq!(
            parse_time_to_utc("08:00 01/06/2023", now)
                .map(|dt| dt.with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())),
            Ok(DateTime::parse_from_rfc3339("2023-06-01T08:00:00+08:00").unwrap())
        )
    }

    #[test]
    fn test_ill_formed_time() {
        let now = utc8_now();
        assert_eq!(
            parse_time_to_utc("05:12 1/20/1111", now).unwrap_err().kind(),
            ParseErrorKind::OutOfRange
        )
    }
}
