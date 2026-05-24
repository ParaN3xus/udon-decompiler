use time::macros::date;
use time::{Date, Duration, Month, PrimitiveDateTime, Time};

pub(super) const TICKS_PER_MILLISECOND: i64 = 10_000;
const NANOS_PER_TICK: i128 = 100;

#[derive(Debug, Clone, Copy)]
pub(super) struct DateTimeParts {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TimeSpanParts {
    pub days: i64,
    pub hours: i64,
    pub minutes: i64,
    pub seconds: i64,
    pub milliseconds: i64,
}

pub(super) fn timespan_parts_from_ticks(ticks: i64) -> Option<TimeSpanParts> {
    if ticks % TICKS_PER_MILLISECOND != 0 {
        return None;
    }
    let sign = if ticks < 0 { -1i64 } else { 1i64 };
    let abs_ticks = ticks.unsigned_abs();
    Some(TimeSpanParts {
        days: i64::try_from(abs_ticks / ticks_per_day() as u64).ok()? * sign,
        hours: i64::try_from((abs_ticks % ticks_per_day() as u64) / ticks_per_hour() as u64)
            .ok()?
            * sign,
        minutes: i64::try_from((abs_ticks % ticks_per_hour() as u64) / ticks_per_minute() as u64)
            .ok()?
            * sign,
        seconds: i64::try_from((abs_ticks % ticks_per_minute() as u64) / ticks_per_second() as u64)
            .ok()?
            * sign,
        milliseconds: i64::try_from(
            (abs_ticks % ticks_per_second() as u64) / TICKS_PER_MILLISECOND as u64,
        )
        .ok()?
            * sign,
    })
}

pub(super) fn timespan_ticks_from_components(
    days: i32,
    hours: i32,
    minutes: i32,
    seconds: i32,
    milliseconds: i32,
) -> Option<i64> {
    let duration = Duration::days(i64::from(days))
        .checked_add(Duration::hours(i64::from(hours)))?
        .checked_add(Duration::minutes(i64::from(minutes)))?
        .checked_add(Duration::seconds(i64::from(seconds)))?
        .checked_add(Duration::milliseconds(i64::from(milliseconds)))?;
    duration
        .whole_nanoseconds()
        .checked_div(NANOS_PER_TICK)?
        .try_into()
        .ok()
}

pub(super) fn datetime_parts_from_ticks(ticks: i64) -> Option<DateTimeParts> {
    if ticks < 0 {
        return None;
    }
    let datetime = dotnet_epoch().checked_add(Duration::nanoseconds_i128(
        i128::from(ticks).checked_mul(NANOS_PER_TICK)?,
    ))?;
    Some(parts_from_datetime(datetime))
}

pub(super) fn datetime_ticks_from_components(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
    millisecond: i32,
) -> Option<i64> {
    let month = Month::try_from(u8::try_from(month).ok()?).ok()?;
    let date = Date::from_calendar_date(year, month, u8::try_from(day).ok()?).ok()?;
    let time = Time::from_hms_milli(
        u8::try_from(hour).ok()?,
        u8::try_from(minute).ok()?,
        u8::try_from(second).ok()?,
        u16::try_from(millisecond).ok()?,
    )
    .ok()?;
    let duration = PrimitiveDateTime::new(date, time) - dotnet_epoch();
    duration
        .whole_nanoseconds()
        .checked_div(NANOS_PER_TICK)?
        .try_into()
        .ok()
}

pub(super) fn parse_datetimeoffset_storage_text(text: &str) -> Option<DateTimeParts> {
    let (date, rest) = text.split_once('T')?;
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i32>().ok()?;
    let month = date_parts.next()?.parse::<i32>().ok()?;
    let day = date_parts.next()?.parse::<i32>().ok()?;
    if date_parts.next().is_some() {
        return None;
    }

    let time = rest.strip_suffix("+00:00")?;
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<i32>().ok()?;
    let minute = time_parts.next()?.parse::<i32>().ok()?;
    let second_fraction = time_parts.next()?;
    if time_parts.next().is_some() {
        return None;
    }
    let (second_text, fraction_text) = second_fraction.split_once('.')?;
    let second = second_text.parse::<i32>().ok()?;
    let fraction = fraction_text.parse::<u32>().ok()?;
    let millisecond = i32::try_from(fraction / 10_000).ok()?;
    datetime_ticks_from_components(year, month, day, hour, minute, second, millisecond)?;
    Some(DateTimeParts {
        year,
        month: u8::try_from(month).ok()?,
        day: u8::try_from(day).ok()?,
        hour: u8::try_from(hour).ok()?,
        minute: u8::try_from(minute).ok()?,
        second: u8::try_from(second).ok()?,
        millisecond: u16::try_from(millisecond).ok()?,
    })
}

fn dotnet_epoch() -> PrimitiveDateTime {
    date!(0001 - 01 - 01).midnight()
}

fn parts_from_datetime(datetime: PrimitiveDateTime) -> DateTimeParts {
    DateTimeParts {
        year: datetime.year(),
        month: datetime.month() as u8,
        day: datetime.day(),
        hour: datetime.hour(),
        minute: datetime.minute(),
        second: datetime.second(),
        millisecond: datetime.millisecond(),
    }
}

const fn ticks_per_second() -> i64 {
    TICKS_PER_MILLISECOND * 1000
}

const fn ticks_per_minute() -> i64 {
    ticks_per_second() * 60
}

const fn ticks_per_hour() -> i64 {
    ticks_per_minute() * 60
}

const fn ticks_per_day() -> i64 {
    ticks_per_hour() * 24
}
