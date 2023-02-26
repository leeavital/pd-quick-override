use std::borrow::Borrow;
use std::cmp::min;
use std::{error::Error, fmt::Display, ops::Add};

#[allow(unused)]
use chrono::TimeZone;
use chrono::{Duration, DurationRound, Timelike, Datelike};

use chrono::{DateTime};
use chrono_tz::Tz;


struct Parse<'a, T> {
    rest: &'a str,
    result: T,
}

#[derive(PartialEq, Eq)]
enum Meridiem {
    Am, Pm 
}

// ranges come in the following forms:
// <full-range> := <date> , <time> - <time>
//                 | <date> <time> - <date> <time>
//                 | <date> - <date> , <time-range>

//  <date> := today
//             | tomorrow
//             |  <D:month>/<D:day>

//  <time> :=  <D:hour> (am | pm)
//             <D:hour>:<D:minute> (am | pm)
pub fn parse(
    now: &DateTime<Tz>,
    range_str: &str,
) -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {

    let lowered_string = range_str.to_lowercase();

    // TODO: no date clone?

    let date_parse = parse_date(now.clone(), &lowered_string)?;
    let comma_parse = parse_literal(date_parse.rest , ",")?;
    let start_time_parse = parse_time(date_parse.result, comma_parse.rest)?;
    let hyphen_parse = parse_literal(start_time_parse.rest, "-")?;
    let end_time_parse = parse_time(date_parse.result, hyphen_parse.rest)?;

    Ok((start_time_parse.result, end_time_parse.result))
}

fn parse_date(now: DateTime<Tz>, source: &str) -> Result<Parse<DateTime<Tz>>, ParseError> {

    if let Ok(parse) = parse_literal(source, "today") {
        let today = now.duration_trunc(Duration::days(1)).unwrap();
        return Ok(Parse { rest: parse.rest, result: today });
    }

    if let Ok(parse) = parse_literal(source, "tomorrow") {
        let today = now.duration_trunc(Duration::days(1)).unwrap();
        let tomorrow= today.add(Duration::days(1));
        return Ok(Parse { rest: parse.rest, result: tomorrow });
    } 

    if let Ok(parse) = parse_month_day_date(now, source) {
        return Ok(parse);
    }

    return Err(ParseError { reason: 
        ParseErrorReason::UnrecognizedDate,
        source: Some(String::from(source)),
    });
}

fn parse_month_day_date(now: DateTime<Tz>, source: &str) -> Result<Parse<DateTime<Tz>>, ParseError> {

    let month_parse = parse_number(source)?;
    let slash_parse = parse_literal(month_parse.rest, "/")?;
    let date_parse = parse_number(slash_parse.rest)?;

    // TODO: year

    let date = now.duration_trunc(Duration::days(1)).unwrap()
        .with_month(month_parse.result).unwrap()
        .with_day(date_parse.result)
        .unwrap();

    Ok(Parse { rest: date_parse.rest, result: date })
}

fn parse_time(base: DateTime<Tz>, source: &str) -> Result<Parse<DateTime<Tz>>, ParseError> {
    
    let hour_parse = parse_number(source)?;
    let mut rest = hour_parse.rest;

    let colon_parse = parse_literal(rest, ":");
    let mut minute = Some(0);
    if colon_parse.is_ok() {
        rest = colon_parse.unwrap().rest;
        let minute_parse = parse_number(rest)?;
        
        minute = Some(minute_parse.result);
        rest = minute_parse.rest;
    }


    let mut meridiem = Meridiem::Am;
    if let Ok(meridiem_parse) = parse_meridiem(rest) {
        meridiem = meridiem_parse.result;
        rest = meridiem_parse.rest; 
    }

    let mut hour = hour_parse.result;
    if hour == 12 && meridiem == Meridiem::Am {
        hour = 0;
    }

    if hour != 12 && meridiem == Meridiem::Pm {
        hour += 12;
    }

    // TODO: check bounds

    let time = base.with_hour(hour)
        .unwrap()
        .with_minute(minute.unwrap_or(0))
        .unwrap();

    return Ok(Parse { rest: rest, result: time });
}


fn parse_number<'a>(source: &'a str) -> Result<Parse<'a, u32>, ParseError> {
    let schars = source.chars().take_while(|x| x.is_numeric()).count();
    if schars == 0 {
        return Err(ParseError { reason: ParseErrorReason::IllegalTime, source: Some(String::from(source)) });
    }

    let n = source[0..schars].parse::<u32>();

    return Ok(Parse { rest: &source[schars..source.len()], result: n.unwrap() });
}

fn parse_literal<'a>(source: &'a str, literal: &str) -> Result<Parse<'a, ()>, ParseError> {
    let t1 = source.trim_start_matches(' ');
    if !t1.starts_with(literal) {
        return Err(ParseError { reason: ParseErrorReason::IllegalTime, source: Some(String::from(t1)) });
    }

    let t2 = t1.strip_prefix(literal).unwrap();
    let t3 = t2.trim_start_matches(' ');

    Ok(Parse{
        rest: t3,
        result: (),
    })
}


fn parse_meridiem(source: &str) -> Result<Parse<Meridiem>, ParseError> {

    if let Ok(parse) = parse_literal(source, "am") {
        return Ok(Parse { rest: parse.rest, result: Meridiem::Am });
    }

    if let Ok(parse) = parse_literal(source, "pm") {
        return Ok(Parse { rest: parse.rest, result: Meridiem::Pm });
    }

    return Err(ParseError { 
        reason: ParseErrorReason::Other, 
        source: Some(String::from(source)),
    });
}


// parse a string, of the form 10am-10pm into one date range
fn get_single_range_with_base(
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

    let lowered = timestr.to_lowercase();

    let r = regex::Regex::new(r"(?P<h>\d\d?)(:(?P<m>\d\d))?\s*(?P<me>am|pm)").expect("could not compile");
    let caps = r.captures(lowered.as_str()).ok_or(error)?;

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

        // weird capitalization
        run_test("today, 11Am-12PM",
            Utc.with_ymd_and_hms(2023, 2, 11, 16, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 11, 17, 0, 0));

    }


    #[test]
    fn test_parse_number() {
        let source = " - 10";

        let hyphen_parse = parse_literal(source, "-").expect("expected hyphen to parse");
        let n_parse  = parse_number(hyphen_parse.rest).expect("expected number to parse");

        assert_eq!(n_parse.result, 10);

        let tz: Tz = "America/New_York".parse().unwrap();
        let now = tz.with_ymd_and_hms(2023, 2, 11, 12, 0, 0).unwrap();
        parse_date(now, "today").expect("expected to parse date");
        parse_date(now, "10/30").expect("expected date to parse");
        parse_time(now, "10:30 am").expect("expected date to parse");
    }
}
