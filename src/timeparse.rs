use std::{error::Error, fmt::Display, ops::Add};

#[allow(unused)]
use chrono::TimeZone;
use chrono::{Duration, DurationRound, Timelike};

use chrono::{DateTime};
use chrono_tz::Tz;

// TODO: use references for DateTime wherever possible?

pub fn parse(
    now: &DateTime<Tz>,
    range_str: &str,
) -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {

    let range_parts: Vec<&str> = range_str.split(',').collect();

    if range_parts.len() == 2 {
        let base = get_base_date(now, range_parts[0])?;

        return get_ranges_with_base(base, range_parts[1]);
    }

    Err(ParseError {
        reason: ParseErrorReason::Other,
        source: Some(String::from(range_str)),
    })
}

fn get_ranges_with_base(
    base: DateTime<Tz>,
    range_parts: &str,
) -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {
    if range_parts.contains('-') {
        let parts: Vec<&str> = range_parts.split('-').collect();
        let start = parse_single_time(base, parts[0])?;
        let end = parse_single_time(base, parts[1])?;

        return Ok((start, end));
    }

    Err(ParseError {
        reason: ParseErrorReason::UnrecognizedTimeRange,
        source: Some(String::from(range_parts)),
    })
}

fn parse_single_time(base: DateTime<Tz>, timestr: &str) -> Result<DateTime<Tz>, ParseError> {
    let error = ParseError {
        reason: ParseErrorReason::IllegalTime,
        source: Some(String::from(timestr)),
    };

    let r = regex::Regex::new(r"(?P<h>\d\d?)(:(?P<m>\d\d))?\s*(?P<me>am|pm)").expect("could not compile");
    let caps = r.captures(timestr).ok_or(error)?;

    let mut hour: u32 = caps.name("h").unwrap().as_str().parse().unwrap();

    match caps.name("me").map(|meridiem| meridiem.as_str()) {
        Some("am") => {
            if hour == 12 {
                hour = 0
            }
        },
        Some("pm") => {
            if hour != 12 {
                hour += 12
            }
        },
        None => unreachable!("prevented by regex (none)"),
        Some(e) => unreachable!("prevented by regex {e}"),
    }


    let mut minute = 0;
    if let Some(minute_s) = caps.name("m") {
        minute = minute_s.as_str().trim_start_matches(':').parse().unwrap();
    }


    Ok(base.with_hour(hour).unwrap().with_minute(minute).unwrap())
}

fn get_base_date(now: &DateTime<Tz>, range_str: &str) -> Result<DateTime<Tz>, ParseError> {
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

    Err(ParseError {
        reason: ParseErrorReason::UnrecognizedDate,
        source: Some(String::from(range_str)),
    })
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

        if let Some(s) = self.source.as_ref() {
            f.write_fmt(format_args!(" on input {:?}", s))?
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
        
        // UCT-5
        let now = tz.with_ymd_and_hms(2023, 2, 11, 12, 0, 0).unwrap();

        let run_test =
            |s: &str, from: LocalResult<DateTime<Utc>>, to: LocalResult<DateTime<Utc>>| {
                let (parsed_from, parsed_to) =
                    parse(&now, s).expect(format!("expected to parse {:?}", s).as_str());
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

        run_test("today, 11am-12pm",
            Utc.with_ymd_and_hms(2023, 2, 11, 16, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 11, 17, 0, 0));

        run_test("today, 12am-1am",
            Utc.with_ymd_and_hms(2023, 2, 11, 5, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 11, 6, 0, 0));

        run_test("today, 12:00am-01:30am",
            Utc.with_ymd_and_hms(2023, 2, 11, 5, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 11, 6, 30, 0));

    }
}
