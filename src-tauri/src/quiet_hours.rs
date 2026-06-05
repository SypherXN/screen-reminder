use chrono::{DateTime, Local, NaiveTime, Utc};

use crate::models::AppSettings;

pub fn is_quiet_hours(settings: &AppSettings, now: DateTime<Utc>) -> bool {
    if !settings.quiet_hours_enabled {
        return false;
    }

    let start = parse_time(&settings.quiet_hours_start);
    let end = parse_time(&settings.quiet_hours_end);
    let (Some(start), Some(end)) = (start, end) else {
        return false;
    };

    let local = now.with_timezone(&Local);
    let current = local.time();

    if start <= end {
        current >= start && current < end
    } else {
        // Overnight range, e.g. 22:00 -> 07:00
        current >= start || current < end
    }
}

fn parse_time(value: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(value, "%H:%M")
        .ok()
        .or_else(|| NaiveTime::parse_from_str(value, "%H:%M:%S").ok())
}
