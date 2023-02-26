use std::{ops::Add};
use thiserror::Error;

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

    if let Ok((start, end)) = parse_single_multi_day_range(now.clone(), &lowered_string) {
        return Ok((start, end));
    }

    parse_single_day_range(*now, &lowered_string)
}


fn parse_single_day_range(
    now: DateTime<Tz>,
    source: &str,
)  -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {
    let date_parse = parse_date(now, source)?;
    let comma_parse = parse_literal(date_parse.rest , ",")?;
    let start_time_parse = parse_time(date_parse.result, comma_parse.rest)?;
    let hyphen_parse = parse_literal(start_time_parse.rest, "-")?;
    let end_time_parse = parse_time(date_parse.result, hyphen_parse.rest)?;
    Ok((start_time_parse.result, end_time_parse.result))
}

fn parse_single_multi_day_range(
    now: DateTime<Tz>,
    source: &str,
)  -> Result<(DateTime<Tz>, DateTime<Tz>), ParseError> {
    let start_date_parse = parse_date(now, source)?;
    let start_time_parse = parse_time(start_date_parse.result, start_date_parse.rest)?;
    let hyphen_parse = parse_literal(start_time_parse.rest, "-")?;
    let end_date_parse = parse_date(now, hyphen_parse.rest)?;
    let end_time_parse = parse_time(end_date_parse.result, end_date_parse.rest)?;
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

    Err(ParseError::UnrecognizedDate(String::from(source)))
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

    let mut minute = Some(0);
    if let Ok(colon_parse) = parse_literal(rest, ":"){
        rest = colon_parse.rest;
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

    let time = base.with_hour(hour)
        .unwrap()
        .with_minute(minute.unwrap_or(0))
        .unwrap();

    Ok(Parse { rest, result: time })
}

fn parse_meridiem(source: &str) -> Result<Parse<Meridiem>, ParseError> {
    if let Ok(parse) = parse_literal(source, "am") {
        return Ok(Parse { rest: parse.rest, result: Meridiem::Am });
    }

    if let Ok(parse) = parse_literal(source, "pm") {
        return Ok(Parse { rest: parse.rest, result: Meridiem::Pm });
    }

    Err(ParseError::IllegalMeridiem(String::from(source)))
}

fn parse_number(source: &str) -> Result<Parse<u32>, ParseError> {
    let schars = source.chars().take_while(|x| x.is_numeric()).count();
    if schars == 0 {
        return Err(ParseError::ExpectedNumber(source.to_string()));
    }

    let n = source[0..schars].parse::<u32>();

    Ok(Parse { rest: &source[schars..source.len()], result: n.unwrap() })
}

fn parse_literal<'a>(source: &'a str, literal: &str) -> Result<Parse<'a, ()>, ParseError> {
    let t1 = source.trim_start_matches(' ');
    if !t1.starts_with(literal) {
        return  Err(ParseError::ExpectedLiteral(literal.to_string(), t1.to_string()));
    }

    let t2 = t1.strip_prefix(literal).unwrap();
    let t3 = t2.trim_start_matches(' ');

    Ok(Parse{
        rest: t3,
        result: (),
    })
}


#[derive(Debug, Error)]
pub enum ParseError {
    #[error("expected tomorrow/today/date, but got {0}")]
    UnrecognizedDate(String),

    #[error("illegal value for meridiem, expected am/pm, but got {0}")]
    IllegalMeridiem(String),

    #[error("expected literal {0} but got {1}")]
    ExpectedLiteral(String, String),

    #[error("expected number, got {0}")]
    ExpectedNumber(String),
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
                    parse(&now, s).unwrap_or_else(|_| panic!("expected to parse {:?}", s));
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


        // multi day span
        run_test("today 10am - tomorrow 2pm",
            Utc.with_ymd_and_hms(2023, 2, 11, 15, 0, 0),
            Utc.with_ymd_and_hms(2023, 2, 12, 19, 0, 0))

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
