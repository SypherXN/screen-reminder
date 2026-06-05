use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use reqwest::Client;

use crate::calendar::{parse_ical_datetime, reminder_from_parts, CalendarProvider, SyncResult};
use crate::models::{CalendarAccount, CaldavConnectRequest};
use crate::secrets;

pub struct CaldavProvider {
    client: Client,
}

impl CaldavProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    pub fn connect(request: CaldavConnectRequest) -> Result<CalendarAccount> {
        let account_id = crate::storage::new_id();
        secrets::store_password(&account_id, &request.password)?;

        let account = CalendarAccount {
            id: account_id,
            source: "caldav".to_string(),
            display_name: request.display_name,
            email: Some(request.username.clone()),
            sync_token: None,
            caldav_url: Some(normalize_url(&request.server_url)),
            caldav_username: Some(request.username),
            connected_at: Utc::now(),
            style_overrides: None,
        };

        Ok(account)
    }

    fn calendar_url(account: &CalendarAccount) -> Result<String> {
        account
            .caldav_url
            .clone()
            .context("missing caldav url")
    }

    fn username(account: &CalendarAccount) -> Result<String> {
        account
            .caldav_username
            .clone()
            .context("missing caldav username")
    }

    fn password(&self, account: &CalendarAccount) -> Result<String> {
        secrets::load_password(&account.id)?.context("missing caldav password")
    }
}

impl CalendarProvider for CaldavProvider {
    fn source(&self) -> &'static str {
        "caldav"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        tauri::async_runtime::block_on(self.sync_async(account))
    }
}

impl CaldavProvider {
    async fn sync_async(&self, account: &CalendarAccount) -> Result<SyncResult> {
        let base_url = Self::calendar_url(account)?;
        let username = Self::username(account)?;
        let password = self.password(account)?;

        let start = Utc::now();
        let end = Utc::now() + Duration::days(30);

        let report_body = format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\" ?>
<C:calendar-query xmlns:D=\"DAV:\" xmlns:C=\"urn:ietf:params:xml:ns:caldav\">
  <D:prop>
    <D:getetag/>
    <C:calendar-data/>
  </D:prop>
  <C:filter>
    <C:comp-filter name=\"VCALENDAR\">
      <C:comp-filter name=\"VEVENT\">
        <C:time-range start=\"{}Z\" end=\"{}Z\"/>
      </C:comp-filter>
    </C:comp-filter>
  </C:filter>
</C:calendar-query>",
            start.format("%Y%m%dT%H%M%S"),
            end.format("%Y%m%dT%H%M%S")
        );

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"REPORT")?, &base_url)
            .basic_auth(username, Some(password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(report_body)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let reminders = parse_caldav_response(account, &response)?;
        Ok(SyncResult {
            reminders,
            sync_token: None,
        })
    }
}

fn normalize_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.ends_with("/calendars") || trimmed.contains("/calendar") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/calendars/user")
    }
}

fn parse_caldav_response(account: &CalendarAccount, xml: &str) -> Result<Vec<crate::models::ReminderEvent>> {
    let mut reminders = Vec::new();
    for chunk in xml.split("<C:calendar-data>").skip(1) {
        let ical = chunk
            .split("</C:calendar-data>")
            .next()
            .unwrap_or("")
            .trim();
        if ical.is_empty() {
            continue;
        }
        reminders.extend(parse_ical_events(account, ical)?);
    }
    Ok(reminders)
}

fn parse_ical_events(
    account: &CalendarAccount,
    ical: &str,
) -> Result<Vec<crate::models::ReminderEvent>> {
    let mut reminders = Vec::new();
    for block in ical.split("BEGIN:VEVENT") {
        if !block.contains("END:VEVENT") {
            continue;
        }
        let event_text = block.split("END:VEVENT").next().unwrap_or(block);
        let uid = extract_prop(event_text, "UID").unwrap_or_else(|| crate::storage::new_id());
        let summary = extract_prop(event_text, "SUMMARY").unwrap_or_else(|| "Untitled event".to_string());
        let location = extract_prop(event_text, "LOCATION");
        let url = extract_prop(event_text, "URL");
        let start_raw = extract_prop(event_text, "DTSTART").unwrap_or_default();
        let start = parse_ical_datetime(&start_raw).unwrap_or_else(|_| Utc::now());

        let mut minutes_before = 15i64;
        for alarm in event_text.split("BEGIN:VALARM") {
            if let Some(alarm_text) = alarm.split("END:VALARM").next() {
                if let Some(trigger) = extract_prop(alarm_text, "TRIGGER") {
                    if let Some(parsed) = parse_trigger(&trigger) {
                        minutes_before = parsed;
                        break;
                    }
                }
            }
        }

        reminders.push(reminder_from_parts(
            account,
            &uid,
            &summary,
            start,
            minutes_before,
            location,
            url,
        ));
    }
    Ok(reminders)
}

fn extract_prop(block: &str, key: &str) -> Option<String> {
    for line in block.lines() {
        let line = line.trim();
        if line.starts_with(key) {
            if let Some(value) = line.split_once(':').map(|(_, v)| v.trim().to_string()) {
                return Some(value);
            }
        }
    }
    None
}

fn parse_trigger(trigger: &str) -> Option<i64> {
    if trigger.starts_with("-PT") && trigger.ends_with('M') {
        trigger
            .trim_start_matches("-PT")
            .trim_end_matches('M')
            .parse()
            .ok()
    } else if trigger.starts_with("-PT") && trigger.ends_with('H') {
        trigger
            .trim_start_matches("-PT")
            .trim_end_matches('H')
            .parse::<i64>()
            .ok()
            .map(|h| h * 60)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_event_with_alarm() {
        let account = CalendarAccount {
            id: "a".to_string(),
            source: "caldav".to_string(),
            display_name: "Test".to_string(),
            email: None,
            sync_token: None,
            caldav_url: None,
            caldav_username: None,
            connected_at: Utc::now(),
            style_overrides: None,
        };
        let ical = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:1\nSUMMARY:Meet\nDTSTART:20260101T100000Z\nBEGIN:VALARM\nTRIGGER:-PT15M\nEND:VALARM\nEND:VEVENT\nEND:VCALENDAR";
        let events = parse_ical_events(&account, ical).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Meet");
    }
}
