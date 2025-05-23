use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use std::convert::TryInto;
use std::time::{Duration, UNIX_EPOCH};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DateTimeError {
    #[error("Failed to parse (date-)time")]
    DateTimeParseFailed(#[from] chrono::format::ParseError),
    #[error("Conversion of date-time failed")]
    DateTimeConversionFailed,
    #[error("Failed to parse (date-)time from string")]
    StringParseError,
    #[error("Found invalid date")]
    InvalidDateError,
}

/// Convert NaiveDate to DateTime at the given hour and convert to local time zone
/// Assuming local time zone if zone is not given
pub fn naive_date_to_date_time(
    date: &NaiveDate,
    hour: u32,
    zone: Option<String>,
) -> Result<DateTime<Local>, DateTimeError> {
    let time = if hour >= 24 {
        date.and_hms_milli_opt(23, 59, 59, 999).unwrap()
    } else {
        date.and_hms_milli_opt(hour, 0, 0, 0).unwrap()
    };
    let time = match zone {
        None => Local
            .from_local_datetime(&time)
            .single()
            .ok_or(DateTimeError::DateTimeConversionFailed)?,
        Some(zone) => {
            let tz: Tz = zone.parse().map_err(|_| DateTimeError::StringParseError)?;
            let date_time = tz
                .from_local_datetime(&time)
                .single()
                .ok_or(DateTimeError::DateTimeConversionFailed)?;
            date_time.with_timezone(&Local)
        }
    };
    Ok(time)
}

/// Create Local time set is given as UNIX epoch timestamp (i.e seconds since 1st Jan 1970)
pub fn unix_to_date_time(seconds: u64) -> DateTime<Local> {
    // Creates a new SystemTime from the specified number of whole seconds
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    // Create DateTime from SystemTime
    DateTime::<Local>::from(d)
}

/// Create Local time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the format American weird format `%m-%d-%Y`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str_american(
    date_str: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<DateTime<Local>, DateTimeError> {
    date_time_from_str(date_str, "%m-%d-%Y", hour, zone)
}

/// Create Local time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the format `%Y-%m-%d`
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str_standard(
    date_str: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<DateTime<Local>, DateTimeError> {
    date_time_from_str(date_str, "%F", hour, zone)
}

/// Create Local time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_time_from_str(
    date_str: &str,
    format: &str,
    hour: u32,
    zone: Option<String>,
) -> Result<DateTime<Local>, DateTimeError> {
    let date = NaiveDate::parse_from_str(date_str, format)?;
    naive_date_to_date_time(&date, hour, zone)
}

/// Create Local time from NaiveDate string
/// The following assumptions are made:
/// 0. Date is given in the provided format
/// 1. Date is the date in local time zone if zone is non, otherwise zone is the time zone
/// 2. Hour is set the given hour parameter
/// 3. Minutes, seconds and milliseconds are set to zero
pub fn date_from_str(date_str: &str, format: &str) -> Result<NaiveDate, DateTimeError> {
    Ok(NaiveDate::parse_from_str(date_str, format)?)
}

/// Convert string with added time zone (by default 0) to DateTime<Local>
pub fn to_time(time: &str, zone: i32) -> Result<DateTime<Local>, DateTimeError> {
    // sqlx strips time zone, just add it here again
    let time = format!("{}{:+05}", time, zone);
    let time = DateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S%.3f%z")?;
    let time: DateTime<Local> = DateTime::from(time);
    Ok(time)
}

/// Given a date and time construct a Local DateTime, assuming that
/// the date belongs to local time zone
pub fn make_time(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> Option<DateTime<Local>> {
    let time: NaiveDateTime =
        NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;
    Local.from_local_datetime(&time).single()
}

pub fn to_date(date: NaiveDate) -> Result<time::Date, DateTimeError> {
    time::Date::from_calendar_date(
        date.year(),
        (date.month() as u8)
            .try_into()
            .map_err(|_| DateTimeError::InvalidDateError)?,
        date.day() as u8,
    )
    .map_err(|_| DateTimeError::InvalidDateError)
}

pub fn from_date(date: time::Date) -> Result<NaiveDate, DateTimeError> {
    NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day().into())
        .ok_or(DateTimeError::InvalidDateError)
}

pub fn to_offset_date_time(time: DateTime<Local>) -> Result<time::OffsetDateTime, DateTimeError> {
    time::OffsetDateTime::from_unix_timestamp_nanos(
        time.timestamp_nanos_opt()
            .ok_or(DateTimeError::InvalidDateError)? as i128,
    )
    .map_err(|_| DateTimeError::InvalidDateError)
}

pub fn convert_local_result_to_datetime(
    local_result: chrono::LocalResult<DateTime<Local>>,
) -> Option<DateTime<Local>> {
    match local_result {
        chrono::LocalResult::Single(datetime) => Some(datetime),
        chrono::LocalResult::Ambiguous(datetime1, _) => {
            // choose the earlier one
            Some(datetime1)
        }
        chrono::LocalResult::None => None, // No valid datetime could be found.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_unix_to_date_time() {
        let date: DateTime<Utc> = unix_to_date_time(1587099600).into();
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-04-17 05:00:00", &date_string);
    }

    #[test]
    fn test_date_time_from_str_american() {
        let date = date_time_from_str_american("02-10-2020", 18, None).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }

    #[test]
    fn test_date_date_time_from_str_standard() {
        let date = date_time_from_str_standard("2020-02-10", 18, None).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }

    #[test]
    fn test_date_time_from_str() {
        let date = date_time_from_str("10-2020-02", "%d-%Y-%m", 18, None).unwrap();
        let date: DateTime<Local> = DateTime::from(date);
        let date_string = date.format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!("2020-02-10 18:00:00", &date_string);
    }
}
