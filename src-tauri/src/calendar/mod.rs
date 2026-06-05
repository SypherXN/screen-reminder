use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::models::{CalendarAccount, ReminderEvent};

pub mod apple;
pub mod caldav;
pub mod google;
pub mod google_tasks;
pub mod microsoft_todo;
pub mod outlook;

pub trait CalendarProvider: Send + Sync {
    fn source(&self) -> &'static str;

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult>;
}

pub struct SyncResult {
    pub reminders: Vec<ReminderEvent>,
    pub sync_token: Option<String>,
}

pub fn build_provider(source: &str) -> Result<Box<dyn CalendarProvider>> {
    match source {
        "google" => Ok(Box::new(google::GoogleProvider::new()?)),
        "google_tasks" => Ok(Box::new(google_tasks::GoogleTasksProvider::new()?)),
        "outlook" => Ok(Box::new(outlook::OutlookProvider::new()?)),
        "microsoft_todo" => Ok(Box::new(microsoft_todo::MicrosoftTodoProvider::new()?)),
        "caldav" => Ok(Box::new(caldav::CaldavProvider::new()?)),
        "apple" => Ok(Box::new(apple::AppleProvider::new()?)),
        other => anyhow::bail!("unknown calendar source: {other}"),
    }
}

pub fn reminder_from_parts(
    account: &CalendarAccount,
    external_id: &str,
    title: &str,
    start_time: DateTime<Utc>,
    reminder_minutes_before: i64,
    location: Option<String>,
    url: Option<String>,
) -> ReminderEvent {
    use crate::storage::new_id;

    let reminder_time = start_time - chrono::Duration::minutes(reminder_minutes_before);
    ReminderEvent {
        id: new_id(),
        account_id: account.id.clone(),
        source: account.source.clone(),
        external_id: external_id.to_string(),
        title: title.to_string(),
        start_time,
        reminder_time,
        location,
        url,
        fired_at: None,
        snoozed_until: None,
        dismissed: false,
    }
}

pub fn parse_ical_datetime(value: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(dt.with_timezone(&Utc));
    }
    if value.len() == 8 {
        let naive = chrono::NaiveDate::parse_from_str(value, "%Y%m%d")
            .context("parse date")?
            .and_hms_opt(0, 0, 0)
            .context("build datetime")?;
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    if value.len() >= 15 {
        let naive = chrono::NaiveDateTime::parse_from_str(&value[..15], "%Y%m%dT%H%M%S")
            .context("parse datetime")?;
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }
    anyhow::bail!("unsupported datetime format: {value}")
}
