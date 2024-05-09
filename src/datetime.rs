use chrono::prelude::*;
use chrono::{DateTime, Local};
use std::time::SystemTime;

pub fn current_time_ymdhms(mut fmt: &str) -> String {
    if fmt == "" {
        fmt = "%Y-%m-%d %H:%M:%S";
    }

    let now = Local::now().format(fmt);

    return now.to_string();
}

pub fn chrono_datetime_to_systemtime(dt: DateTime<Utc>) -> SystemTime {
    let st: SystemTime = dt.into();
    return st;
}

pub fn systemtime_to_chrono_datetime(dt: SystemTime) -> DateTime<Utc> {
    let st: DateTime<Utc> = dt.into();
    return st;
}

pub fn timestamp_to_time(timestamp: i64) -> chrono::DateTime<chrono::Utc> {
    let datetime: DateTime<Utc> = Utc.timestamp_opt(timestamp, 0).unwrap();

    return datetime;
}

pub fn get_timezone_offset_of_utc() -> i32 {
    return Local
        .timestamp_opt(0, 0)
        .unwrap()
        .offset()
        .fix()
        .local_minus_utc();
}
