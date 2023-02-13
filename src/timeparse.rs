use std::{error::Error, fmt::Display, ops::Add};

use chrono::{Duration, DurationRound, Timelike};

use chrono::DateTime;
use chrono_tz::Tz;

// TODO: use references for DateTime wherever possible?

#[allow(dead_code)]
pub fn parse(
    tz: Tz,
    now: DateTime<Tz>,
    range_str: &str,
) -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {

    let range_parts: Vec<&str> = range_str.split(",").collect();

    if range_parts.len() == 2 {
        let base = get_base_date(tz, now, range_parts[0])?;

        return get_ranges_with_base(base, range_parts[1]);
    }

    return Err(ParseError {
        reason: ParseErrorReason::Other,
        source: Some(String::from(range_str)),
    });
}

fn get_ranges_with_base(
    base: DateTime<Tz>,
    range_parts: &str,
) -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {
    if range_parts.contains("-") {
        let parts: Vec<&str> = range_parts.split("-").collect();
        let start = parse_single_time(base, parts[0])?;
        let end = parse_single_time(base, parts[1])?;

        return Ok((start, end));
    }

    return Err(ParseError {
        reason: ParseErrorReason::UnrecognizedTimeRange,
        source: Some(String::from(range_parts)),
    });
}

fn parse_single_time(base: DateTime<Tz>, timestr: &str) -> Result<DateTime<Tz>, ParseError> {
    let error = ParseError {
        reason: ParseErrorReason::IllegalTime,
        source: Some(String::from(timestr)),
    };

    let r = regex::Regex::new(r"(?P<h>\d\d?)(:\d\d)?\s*(?P<m>am|pm)?").expect("could not compile");
    let caps = r.captures(timestr).ok_or(error)?;

    let mut hour: u32 = caps.name("h").unwrap().as_str().parse().unwrap();

    match caps.name("m").unwrap().as_str() {
        "am" => {}
        "pm" => hour = hour + 12,
        _ => unreachable!("prevented by regex"),
    }
    return Ok(base.with_hour(hour).unwrap());
}

fn get_base_date(_tz: Tz, now: DateTime<Tz>, range_str: &str) -> Result<DateTime<Tz>, ParseError> {
    if range_str.starts_with("tomorrow") {
        let base_date = now
            .add(chrono::Duration::days(1))
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        return Ok(base_date);
    }

    if range_str.starts_with("today") {
        let base_date = now.duration_trunc(Duration::days(1)).unwrap();

        return Ok(base_date);
    }

    // TODO: exact dates

    return Err(ParseError {
        reason: ParseErrorReason::UnrecognizedDate,
        source: Some(String::from(range_str)),
    });
}

#[derive(Debug)]
pub enum ParseErrorReason {
    UnrecognizedDate,
    UnrecognizedTimeRange,
    IllegalTime,
    Other,
}

#[derive(Debug)]
pub struct ParseError {
    reason: ParseErrorReason,
    source: Option<String>,
}
impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("encountered error code: {:?}", self.reason))?;

        match self.source.as_ref() {
            Some(s) => f.write_fmt(format_args!(" on input {:?}", s))?,
            None => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use chrono::{LocalResult, Utc};

    #[test]
    fn test_me() {
        let tz: Tz = "America/New_York".parse().unwrap();
        let now = tz.with_ymd_and_hms(2023, 2, 11, 12, 0, 0).unwrap();

        let run_test =
            |s: &str, from: LocalResult<DateTime<Utc>>, to: LocalResult<DateTime<Utc>>| {
                let (parsed_from, parsed_to) =
                    parse(tz, now, s).expect(format!("expected to parse {:?}", s).as_str());
                assert_eq!(parsed_from.timestamp(), from.unwrap().timestamp());
                assert_eq!(parsed_to.timestamp(), to.unwrap().timestamp());
            };

        run_test(
            "tomorrow, 10am-10pm",
            Utc.with_ymd_and_hms(2023, 2, 12, 15, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 13, 3, 0, 0),
        );

        run_test(
            "today,9am-9pm",
            Utc.with_ymd_and_hms(2023, 2, 11, 14, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 12, 2, 0, 0),
        );
    }
}
