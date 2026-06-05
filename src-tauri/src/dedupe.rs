use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::models::ReminderEvent;

pub fn dedupe_key(reminder: &ReminderEvent) -> String {
    let title = reminder.title.trim().to_lowercase();
    let start_minute = reminder.start_time.format("%Y-%m-%dT%H:%M").to_string();
    format!("{title}|{start_minute}")
}

pub fn dedupe_reminders(reminders: Vec<ReminderEvent>) -> Vec<ReminderEvent> {
    let mut best: HashMap<String, ReminderEvent> = HashMap::new();

    for reminder in reminders {
        let key = dedupe_key(&reminder);
        best.entry(key)
            .and_modify(|existing| {
                if source_priority(&reminder.source) < source_priority(&existing.source) {
                    *existing = reminder.clone();
                }
            })
            .or_insert(reminder);
    }

    let mut result: Vec<_> = best.into_values().collect();
    result.sort_by_key(|r| r.reminder_time);
    result
}

fn source_priority(source: &str) -> u8 {
    match source {
        "google" => 0,
        "outlook" => 1,
        "google_tasks" => 2,
        "microsoft_todo" => 3,
        "caldav" => 4,
        "apple" => 5,
        _ => 6,
    }
}
