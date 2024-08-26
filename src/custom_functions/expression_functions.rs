use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use crate::{Error, FloatType, Value};
use chrono::{NaiveDateTime,Timelike,Utc, DateTime, Duration, Datelike, TimeZone};
use crate::Error::CustomError;

pub fn is_null<T: Into<Value>>(value: T) ->  Result<Value, Error>  {
    Ok(match value.into() {
        Value::Empty => Value::Int(0),
        v => v,
    })
}pub fn negate<T: Into<Value>>(value: T) ->  Result<Value, Error>  {
    Ok(match value.into() {
        Value::Empty => Value::Empty,
        Value::Int(v) => Value::Int(-v),
        Value::Float(v) => Value::Float(-v),
        Value::Boolean(v) => Value::Boolean(!v),
        v => return Err(Error::CustomError(format!("Cannot negate a value {v:?}"))),
    })
}

pub fn is_null_or<T: Into<Value>,S: Into<Value>>(value: T, alternative: S) ->  Result<Value, Error>  {
    Ok(match value.into() {
        Value::Empty => alternative.into(),
        v => v,
    })
}

pub fn clip_value_to_range<T: Into<Value>, S: Into<Value>>(value: T, constant: S) -> Result<Value, Error> {
    let value: Value = value.into();
    let constant: Value = constant.into();

    let value_as_float = value.as_float_or_none()?.unwrap_or(0.0);
    let constant_as_float = constant.as_float_or_none()?.unwrap_or(0.0);

    let adjusted_value = if value_as_float > constant_as_float {
        constant_as_float
    } else if value_as_float < -constant_as_float {
        -constant_as_float
    } else {
        value_as_float
    };

    Ok(Value::Float(adjusted_value))
}

pub fn fallback_with_range_clipping<P: Into<Value>, L: Into<Value>, C: Into<Value>, D: Into<Value>>(
    should_use_primary: P,
    rel_score_long_temp_ps: L,
    rel_score_long_temp: L,
    range_to_clip: C,
    empty_value: D,
) -> Result<Value, Error> {
    let should_use_primary: Value = should_use_primary.into();
    let rel_score_long_temp_ps: Value = rel_score_long_temp_ps.into();
    let rel_score_long_temp: Value = rel_score_long_temp.into();
    let constant: Value = range_to_clip.into();

    if should_use_primary.as_boolean_or_none()?.unwrap_or(false) {
        if !rel_score_long_temp_ps.is_empty() {
            clip_value_to_range(rel_score_long_temp_ps, constant)
        } else {
            Ok(empty_value.into())
        }
    } else {
        if !rel_score_long_temp.is_empty() {
            clip_value_to_range(rel_score_long_temp, constant)
        } else {
            Ok(Value::Float(0.0)) // Assuming a default return value
        }
    }
}



pub fn abs<T: TryInto<Value>>(value: T) -> Result<Value, Error>
    where <T as TryInto<Value>>::Error: Debug
{

    match value.try_into().map_err(|err| CustomError(format!("{err:?}")))? {
        Value::Float(fl) => { Ok(Value::Float(fl.abs())) }
        Value::Int(nn) => { Ok(Value::Int(nn.abs())) }
        Value::Empty => {Ok(Value::Empty)}
        _ => Err(Error::InvalidArgumentType),
    }
}
pub fn safe_divide<TL: Into<Value>,TR: Into<Value>>(left: TL, right: TR) -> Result<Value, Error> {
    match (left.into(), right.into()) {
        (Value::Float(left), Value::Float(right)) => {
            if right == 0.0 {
                Ok(Value::Empty)
            } else {
                Ok(Value::Float(left / right))
            }
        }
        (Value::Int(left), Value::Int(right)) => {
            if right == 0 {
                Ok(Value::Empty)
            } else {
                Ok(Value::Int(left / right))
            }
        }
        (Value::Float(left), Value::Int(right)) => {
            if right == 0 {
                Ok(Value::Empty)
            } else {
                Ok(Value::Float(left / right as FloatType))
            }
        }
        (Value::Int(left), Value::Float(right)) => {
            if right == 0.0 {
                Ok(Value::Empty)
            } else {
                Ok(Value::Float(left as FloatType / right))
            }
        }
        (_, Value::Empty) => {
            Ok(Value::Empty)
        },
        (Value::Empty,_) => {
            Ok(Value::Empty)
        }

        _ => Err(Error::InvalidArgumentType),
    }
}

impl TryFrom<Result<Value, String>> for Value {
    type Error = String;

    fn try_from(value: Result<Value, String>) -> Result<Self, Self::Error> {
        match value {
            Ok(num) => Ok(num),
            Err(e) => Err(e),
        }
    }
}


pub fn substring<TL: Into<Value>,TR: Into<Value>, TC: Into<Value>>(message: TL, start: TR, len: TC) -> Result<Value, Error> {
    if let Value::String(message) = message.into() {
        // Ensure start is within bounds and len does not exceed the message length
        let start_int = Into::<Value>::into(start).as_int()? as usize;
        let len_int = Into::<Value>::into(len).as_int()? as usize;
        let message = message.into_owned();
        if start_int < message.len()  {
            let end = if start_int + len_int > message.len() { message.len() } else { start_int + len_int };
            let substring = &message[start_int..end];
            return Ok(substring.to_string().into());
        }
    }
    Ok("".to_string().into())
}


pub fn starts_with(message: &Value, prefix: &Value) ->  Result<Value, Error>  {
    if let (Value::String(message), Value::String(prefix)) = (message, prefix) {
        let message = message.ref_into_owned();
        let prefix = prefix.ref_into_owned();
        if message.starts_with(&prefix) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

pub fn ends_with(message: &Value, suffix: &Value) ->  Result<Value, Error>  {
    if let (Value::String(message), Value::String(prefix)) = (message, suffix) {
        let message = message.ref_into_owned();
        let prefix = prefix.ref_into_owned();
        if message.ends_with(&prefix) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

pub fn ternary<TC: Into<Value>,TL: Into<Value>,TR: Into<Value>>(condition: TC, true_value: TL, false_value: TR) -> Result<Value, Error> {
    if let Value::Boolean(cond) = condition.into() {
        if cond {
            return Ok(true_value.into());
        } else {
            return Ok(false_value.into());
        }
    }
    // Return an error if the first parameter is not a boolean
    Err(Error::CustomError("First parameter must be a boolean".to_owned()))
}

fn round_datetime_to_precision(datetime: DateTime<Utc>, precision: &str) -> Result<DateTime<Utc>, crate::Error> {
    Ok(match precision {
        "m1" => datetime.date().and_hms(datetime.hour(), datetime.minute(), 0),
        "m5" => datetime.date().and_hms(datetime.hour(), (datetime.minute() / 5) * 5, 0),
        "m15" => datetime.date().and_hms(datetime.hour(), (datetime.minute() / 15) * 15, 0),
        "m30" => datetime.date().and_hms(datetime.hour(), (datetime.minute() / 30) * 30, 0),
        "h1" => datetime.date().and_hms(datetime.hour(), 0, 0),
        "h4" => datetime.date().and_hms((datetime.hour() / 4) * 4, 0, 0),
        "d1" => datetime.date().and_hms(0, 0, 0),
        "1w" => (datetime - Duration::days(datetime.date().weekday().num_days_from_sunday() as i64)).date().and_hms(0, 0, 0),
        "1M" => datetime.date().with_day(1).unwrap().and_hms(0, 0, 0),
        val => {
            return Err(Error::CustomError(format!("Precision {val} is not recognised")));
        } // If the precision is not recognized, return the original datetime
    })
}

pub fn round_date_to_precision<TL: Into<Value>,TR: Into<Value>>(string: TL, precision: TR) -> Result<Value, crate::Error> {
    if let (Value::String(string), Value::String(precision)) = (string.into(), precision.into()) {
        // Extract the date-time part from the input string
        let string = string.into_owned();
        let precision = precision.into_owned();
        let parts: Vec<&str> = string.split('_').collect();
        let datetime_str = parts.last().ok_or_else(|| Error::InvalidInputString)?;

        let naive_datetime = NaiveDateTime::parse_from_str(datetime_str, "%Y.%m.%d %H:%M:%S")
            .map_err(|_| Error::InvalidDateFormat)?;
        let datetime = Utc.from_utc_datetime(&naive_datetime);
        let rounded_datetime = round_datetime_to_precision(datetime, &precision.to_lowercase())?;
        let mut string1 = parts.iter().take(parts.len() - 1).map(|prt| prt.to_string()).collect::<Vec<String>>().join("_");
        if string1.len() > 0 {
            string1.push_str("_");
        }
        let result = format!("{}{}", string1, rounded_datetime.format("%Y.%m.%d %H:%M:%S").to_string());
        Ok(result.into())
    } else {
        // If arguments are not strings, return an error
        Err(Error::InvalidArgumentType)
    }
}

pub fn max<TL: Into<Value>,TR: Into<Value>>(value1: TL, value2: TR) ->  Result<Value, Error>  {
    let x = value1.into();
    let x1 = value2.into();
    Ok(if x > x1 {
        x
    } else {
        x1
    })
}

pub fn min<TL: Into<Value>,TR: Into<Value>>(value1: TL, value2: TR) -> Result<Value, Error>  {
    let x = value1.into();
    let x1 = value2.into();
    Ok(if x < x1 {
        x
    } else {
        x1
    })
}


mod test{

    use super::*;
    #[test]
    fn test_round_date_to_m1() {
        let input = (
            Value::String("BTCUSD_2024.02.13 10:05:23".into()),
            Value::String("m1".into())
        );
        let expected = Utc.ymd(2024, 2, 13).and_hms(10, 5, 0).format("%Y.%m.%d %H:%M:%S").to_string();
        let result = round_date_to_precision(&input.0, &input.1).unwrap();
        assert_eq!(result, format!("BTCUSD_{}", expected).into());
    }

    #[test]
    fn test_round_date_to_h1() {
        let input = (
            Value::String("BTCUSD_2024.02.13 10:05:23".into()),
            Value::String("h1".into())
        );
        let expected = Utc.ymd(2024, 2, 13).and_hms(10, 0, 0).format("%Y.%m.%d %H:%M:%S").to_string();
        let result = round_date_to_precision(&input.0, &input.1).unwrap();
        assert_eq!(result, format!("BTCUSD_{}", expected).into());
    }

    #[test]
    fn test_round_date_to_1w() {
        let input = (
            Value::String("BTCUSD_2024.02.13 10:05:23".into()),
            Value::String("1w".into())
        );
        // Assuming 2024-02-13 is a Wednesday, rounding to the start of the week (Sunday)
        let expected = Utc.ymd(2024, 2, 11).and_hms(0, 0, 0).format("%Y.%m.%d %H:%M:%S").to_string();
        let result = round_date_to_precision(&input.0, &input.1).unwrap();
        assert_eq!(result, format!("BTCUSD_{}", expected).into());
    }

    #[test]
    fn test_invalid_date_format() {
        let input = (
            Value::String("BTCUSD_ThisIsNotADate".into()),
            Value::String("m1".into())
        );
        let result = round_date_to_precision(&input.0, &input.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_precision() {
        let input = (
            Value::String("BTCUSD_2024.02.13 10:05:00".into()),
            Value::String("m60".into())
        );
        let result = round_date_to_precision(&input.0, &input.1);
        assert!(result.is_err(), "Expected an error for invalid precision");
    }
}
