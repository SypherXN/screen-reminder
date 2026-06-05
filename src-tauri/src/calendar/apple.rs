use anyhow::{Context, Result};
use chrono::Utc;

use crate::calendar::{reminder_from_parts, CalendarProvider, SyncResult};
use crate::models::CalendarAccount;

pub struct AppleProvider;

impl AppleProvider {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn connect() -> Result<CalendarAccount> {
        #[cfg(target_os = "macos")]
        {
            macos_eventkit::request_access()?;
            Ok(CalendarAccount {
                id: crate::storage::new_id(),
                source: "apple".to_string(),
                display_name: "Apple Calendar".to_string(),
                email: None,
                sync_token: None,
                caldav_url: None,
                caldav_username: None,
                connected_at: Utc::now(),
                style_overrides: None,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            anyhow::bail!("Apple Calendar is only available on macOS")
        }
    }
}

impl CalendarProvider for AppleProvider {
    fn source(&self) -> &'static str {
        "apple"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        #[cfg(target_os = "macos")]
        {
            let events = macos_eventkit::fetch_upcoming_events(30)?;
            let mut reminders = Vec::new();
            for event in events {
                let start = event.start;
                let minutes = event.minutes_before.unwrap_or(15).max(0) as i64;
                reminders.push(reminder_from_parts(
                    account,
                    &event.id,
                    &event.title,
                    start,
                    minutes,
                    event.location,
                    event.url,
                ));
            }
            Ok(SyncResult {
                reminders,
                sync_token: None,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            let _ = account;
            Ok(SyncResult {
                reminders: vec![],
                sync_token: None,
            })
        }
    }
}

#[cfg(target_os = "macos")]
mod macos_eventkit {
    use anyhow::{Context, Result};
    use chrono::{DateTime, Local, NaiveDateTime, Utc};
    use std::process::Command;

    const RECORD_SEP: char = '\x1e';
    const FIELD_SEP: char = '\x1f';

    pub struct AppleEvent {
        pub id: String,
        pub title: String,
        pub start: DateTime<Utc>,
        pub minutes_before: Option<i32>,
        pub location: Option<String>,
        pub url: Option<String>,
    }

    pub fn request_access() -> Result<()> {
        let script = r#"
            tell application "Calendar"
                if not running then launch
                try
                    get count of calendars
                    return "ok"
                on error errMsg number errNum
                    return "error:" & errNum & ":" & errMsg
                end try
            end tell
        "#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .context("request calendar access")?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout == "ok" {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Calendar access failed ({stdout}). Grant access in System Settings → Privacy → Calendars. {stderr}"
        )
    }

    pub fn fetch_upcoming_events(days: i64) -> Result<Vec<AppleEvent>> {
        let script = format!(
            r#"
            set recordSep to ASCII character 30
            set fieldSep to ASCII character 31
            set outputLines to {{}}
            set endDate to (current date) + ({days} * days)
            tell application "Calendar"
                repeat with cal in calendars
                    set calEvents to (every event of cal whose start date ≥ (current date) and start date ≤ endDate)
                    repeat with ev in calEvents
                        set eventId to uid of ev as string
                        set eventTitle to summary of ev as string
                        set eventStart to start date of ev
                        set eventLocation to ""
                        try
                            set eventLocation to location of ev as string
                        end try
                        set alarmMinutes to 15
                        try
                            set alarmList to alarms of ev
                            if (count of alarmList) > 0 then
                                set triggerSecs to (trigger interval of item 1 of alarmList) as integer
                                set alarmMinutes to (abs triggerSecs) div 60
                                if alarmMinutes < 0 then set alarmMinutes to 0
                            end if
                        end try
                        set eventUrl to ""
                        try
                            set eventUrl to url of ev as string
                        end try
                        set end of outputLines to (eventId & fieldSep & eventTitle & fieldSep & (eventStart as «class isot» as string) & fieldSep & alarmMinutes & fieldSep & eventLocation & fieldSep & eventUrl)
                    end repeat
                end repeat
            end tell
            set oldDelims to AppleScript's text item delimiters
            set AppleScript's text item delimiters to recordSep
            set resultText to outputLines as string
            set AppleScript's text item delimiters to oldDelims
            return resultText
            "#
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .context("fetch apple calendar events")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Apple Calendar script failed: {stderr}");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_event_payload(&stdout)
    }

    fn parse_event_payload(stdout: &str) -> Result<Vec<AppleEvent>> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() || trimmed == "missing value" {
            return Ok(vec![]);
        }

        let mut events = Vec::new();
        for record in trimmed.split(RECORD_SEP) {
            let record = record.trim();
            if record.is_empty() {
                continue;
            }

            let parts: Vec<&str> = record.split(FIELD_SEP).collect();
            if parts.len() < 4 {
                continue;
            }

            let id = parts[0].trim().to_string();
            let title = parts[1].trim().to_string();
            if id.is_empty() || title.is_empty() {
                continue;
            }

            let start = parse_apple_date(parts[2].trim()).unwrap_or_else(|_| Utc::now());
            let minutes_before = parts[3].trim().parse::<i32>().ok();
            let location = parts
                .get(4)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let url = parts
                .get(5)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            events.push(AppleEvent {
                id,
                title,
                start,
                minutes_before,
                location,
                url,
            });
        }

        Ok(events)
    }

    fn parse_apple_date(value: &str) -> Result<DateTime<Utc>> {
        if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(dt) = chrono::DateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%z") {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
            return Ok(Local
                .from_local_datetime(&naive)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now));
        }
        anyhow::bail!("unsupported apple date: {value}")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parses_record_payload() {
            let sample = format!(
                "abc{FIELD_SEP}Standup{FIELD_SEP}2026-06-05T09:00:00{FIELD_SEP}10{FIELD_SEP}Zoom{FIELD_SEP}"
            );
            let events = parse_event_payload(&sample).unwrap();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].title, "Standup");
            assert_eq!(events[0].minutes_before, Some(10));
        }
    }
}
