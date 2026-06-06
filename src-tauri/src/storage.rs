use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::models::{AccountStyleOverrides, AppSettings, CalendarAccount, CompositionPreset, ReminderEvent, PushSubscription};

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path).context("open database")?;
        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.migrate()?;
        Ok(storage)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS calendar_accounts (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                display_name TEXT NOT NULL,
                email TEXT,
                sync_token TEXT,
                caldav_url TEXT,
                caldav_username TEXT,
                connected_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS reminder_events (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                source TEXT NOT NULL,
                external_id TEXT NOT NULL,
                title TEXT NOT NULL,
                start_time TEXT NOT NULL,
                reminder_time TEXT NOT NULL,
                location TEXT,
                url TEXT,
                fired_at TEXT,
                snoozed_until TEXT,
                dismissed INTEGER NOT NULL DEFAULT 0,
                UNIQUE(source, external_id, reminder_time)
            );

            CREATE TABLE IF NOT EXISTS sync_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS composition_presets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                composition_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS push_subscriptions (
                account_id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                channel_id TEXT NOT NULL,
                resource_id TEXT,
                expiration TEXT NOT NULL
            );
            ",
        )?;

        let _ = conn.execute(
            "ALTER TABLE calendar_accounts ADD COLUMN style_json TEXT",
            [],
        );

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let settings = AppSettings::default();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('app_settings', ?1)",
                params![serde_json::to_string(&settings)?],
            )?;
        }

        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings> {
        let conn = self.conn.lock().unwrap();
        let json: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'app_settings'",
            [],
            |row| row.get(0),
        )?;
        let mut settings: AppSettings = serde_json::from_str(&json)?;
        settings.ensure_composition();
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE settings SET value = ?1 WHERE key = 'app_settings'",
            params![serde_json::to_string(settings)?],
        )?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<CalendarAccount>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source, display_name, email, sync_token, caldav_url, caldav_username, connected_at, style_json
             FROM calendar_accounts ORDER BY connected_at",
        )?;
        let rows = stmt.query_map([], parse_account_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list accounts")
    }

    pub fn get_account(&self, id: &str) -> Result<Option<CalendarAccount>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, source, display_name, email, sync_token, caldav_url, caldav_username, connected_at, style_json
             FROM calendar_accounts WHERE id = ?1",
            params![id],
            parse_account_row,
        )
        .optional()
        .context("get account")
    }

    pub fn find_account_by_source_and_email(
        &self,
        source: &str,
        email: &str,
    ) -> Result<Option<CalendarAccount>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, source, display_name, email, sync_token, caldav_url, caldav_username, connected_at, style_json
             FROM calendar_accounts
             WHERE source = ?1 AND lower(email) = lower(?2)
             LIMIT 1",
            params![source, email],
            parse_account_row,
        )
        .optional()
        .context("find account by source and email")
    }

    pub fn find_caldav_account(
        &self,
        caldav_url: &str,
        username: &str,
    ) -> Result<Option<CalendarAccount>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, source, display_name, email, sync_token, caldav_url, caldav_username, connected_at, style_json
             FROM calendar_accounts
             WHERE source = 'caldav' AND caldav_url = ?1 AND caldav_username = ?2
             LIMIT 1",
            params![caldav_url, username],
            parse_account_row,
        )
        .optional()
        .context("find caldav account")
    }

    pub fn upsert_account(&self, account: &CalendarAccount) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let style_json = account
            .style_overrides
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        conn.execute(
            "INSERT INTO calendar_accounts (id, source, display_name, email, sync_token, caldav_url, caldav_username, connected_at, style_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
               display_name = excluded.display_name,
               email = excluded.email,
               sync_token = excluded.sync_token,
               caldav_url = excluded.caldav_url,
               caldav_username = excluded.caldav_username,
               style_json = excluded.style_json",
            params![
                account.id,
                account.source,
                account.display_name,
                account.email,
                account.sync_token,
                account.caldav_url,
                account.caldav_username,
                account.connected_at.to_rfc3339(),
                style_json,
            ],
        )?;
        Ok(())
    }

    pub fn save_account_style(
        &self,
        account_id: &str,
        style: &AccountStyleOverrides,
    ) -> Result<()> {
        let mut account = self
            .get_account(account_id)?
            .ok_or_else(|| anyhow::anyhow!("account not found"))?;
        account.style_overrides = Some(style.clone());
        self.upsert_account(&account)
    }

    pub fn get_account_style(&self, account_id: &str) -> Result<Option<AccountStyleOverrides>> {
        Ok(self
            .get_account(account_id)?
            .and_then(|account| account.style_overrides))
    }

    pub fn list_composition_presets(&self) -> Result<Vec<CompositionPreset>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, composition_json, created_at FROM composition_presets ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CompositionPreset {
                id: row.get(0)?,
                name: row.get(1)?,
                composition: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                created_at: row
                    .get::<_, String>(3)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list composition presets")
    }

    pub fn save_composition_preset(&self, name: &str, composition: &crate::models::OverlayComposition) -> Result<CompositionPreset> {
        let preset = CompositionPreset {
            id: new_id(),
            name: name.to_string(),
            composition: composition.clone(),
            created_at: Utc::now(),
        };
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO composition_presets (id, name, composition_json, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                preset.id,
                preset.name,
                serde_json::to_string(&preset.composition)?,
                preset.created_at.to_rfc3339(),
            ],
        )?;
        Ok(preset)
    }

    pub fn delete_composition_preset(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM composition_presets WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_composition_preset(&self, id: &str) -> Result<Option<CompositionPreset>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, composition_json, created_at FROM composition_presets WHERE id = ?1",
            params![id],
            |row| {
                Ok(CompositionPreset {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    composition: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    created_at: row
                        .get::<_, String>(3)?
                        .parse()
                        .unwrap_or_else(|_| Utc::now()),
                })
            },
        )
        .optional()
        .context("get composition preset")
    }

    pub fn nearest_reminder_minutes(&self) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let next: Option<String> = conn
            .query_row(
                "SELECT reminder_time FROM reminder_events
                 WHERE dismissed = 0 AND reminder_time > ?1
                 ORDER BY reminder_time ASC LIMIT 1",
                params![now],
                |row| row.get(0),
            )
            .optional()?;

        Ok(next.and_then(|value| {
            value
                .parse::<DateTime<Utc>>()
                .ok()
                .map(|dt| (dt - Utc::now()).num_minutes().max(0))
        }))
    }

    pub fn save_push_subscription(&self, sub: &PushSubscription) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO push_subscriptions (account_id, source, channel_id, resource_id, expiration)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(account_id) DO UPDATE SET
               source = excluded.source,
               channel_id = excluded.channel_id,
               resource_id = excluded.resource_id,
               expiration = excluded.expiration",
            params![
                sub.account_id,
                sub.source,
                sub.channel_id,
                sub.resource_id,
                sub.expiration.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn list_push_subscriptions(&self) -> Result<Vec<PushSubscription>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT account_id, source, channel_id, resource_id, expiration FROM push_subscriptions",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PushSubscription {
                account_id: row.get(0)?,
                source: row.get(1)?,
                channel_id: row.get(2)?,
                resource_id: row.get(3)?,
                expiration: row
                    .get::<_, String>(4)?
                    .parse()
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list push subscriptions")
    }

    pub fn delete_account(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM calendar_accounts WHERE id = ?1", params![id])?;
        conn.execute(
            "DELETE FROM reminder_events WHERE account_id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn replace_reminders_for_account(
        &self,
        account_id: &str,
        reminders: &[ReminderEvent],
    ) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM reminder_events WHERE account_id = ?1",
            params![account_id],
        )?;
        drop(conn);
        self.upsert_reminders(reminders)
    }

    pub fn upsert_reminders(&self, reminders: &[ReminderEvent]) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let mut count = 0;
        for reminder in reminders {
            conn.execute(
                "INSERT INTO reminder_events
                 (id, account_id, source, external_id, title, start_time, reminder_time, location, url, fired_at, snoozed_until, dismissed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT(source, external_id, reminder_time) DO UPDATE SET
                   title = excluded.title,
                   start_time = excluded.start_time,
                   location = excluded.location,
                   url = excluded.url",
                params![
                    reminder.id,
                    reminder.account_id,
                    reminder.source,
                    reminder.external_id,
                    reminder.title,
                    reminder.start_time.to_rfc3339(),
                    reminder.reminder_time.to_rfc3339(),
                    reminder.location,
                    reminder.url,
                    reminder.fired_at.map(|t| t.to_rfc3339()),
                    reminder.snoozed_until.map(|t| t.to_rfc3339()),
                    reminder.dismissed as i32,
                ],
            )?;
            count += 1;
        }
        Ok(count)
    }

    pub fn due_reminders(
        &self,
        now: DateTime<Utc>,
        dedupe: bool,
    ) -> Result<Vec<ReminderEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, source, external_id, title, start_time, reminder_time, location, url, fired_at, snoozed_until, dismissed
             FROM reminder_events
             WHERE dismissed = 0
               AND (fired_at IS NULL OR snoozed_until IS NOT NULL)
               AND (
                 (snoozed_until IS NOT NULL AND snoozed_until <= ?1)
                 OR (snoozed_until IS NULL AND reminder_time <= ?1)
               )",
        )?;
        let now_str = now.to_rfc3339();
        let rows = stmt.query_map(params![now_str], |row| parse_reminder_row(row))?;
        let mut due = rows
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("due reminders")?;

        if dedupe {
            due = crate::dedupe::dedupe_reminders(due);
        }

        Ok(due)
    }

    pub fn get_reminder(&self, id: &str) -> Result<Option<ReminderEvent>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, account_id, source, external_id, title, start_time, reminder_time, location, url, fired_at, snoozed_until, dismissed
             FROM reminder_events WHERE id = ?1",
            params![id],
            parse_reminder_row,
        )
        .optional()
        .context("get reminder")
    }

    pub fn mark_fired(&self, id: &str, at: DateTime<Utc>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE reminder_events SET fired_at = ?1, snoozed_until = NULL WHERE id = ?2",
            params![at.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn dismiss_reminder(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE reminder_events SET dismissed = 1, snoozed_until = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn snooze_reminder(&self, id: &str, until: DateTime<Utc>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE reminder_events SET snoozed_until = ?1, fired_at = NULL WHERE id = ?2",
            params![until.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn reminder_count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM reminder_events WHERE dismissed = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn list_upcoming_reminders(
        &self,
        now: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<ReminderEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, account_id, source, external_id, title, start_time, reminder_time, location, url, fired_at, snoozed_until, dismissed
             FROM reminder_events
             WHERE dismissed = 0
               AND (
                 start_time >= ?1
                 OR reminder_time >= ?1
                 OR (snoozed_until IS NOT NULL AND snoozed_until >= ?1)
               )
             ORDER BY start_time ASC
             LIMIT ?2",
        )?;
        let now_str = now.to_rfc3339();
        let rows = stmt.query_map(params![now_str, limit as i64], parse_reminder_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("list upcoming reminders")
    }

    pub fn set_sync_meta(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sync_meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_sync_meta(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row("SELECT value FROM sync_meta WHERE key = ?1", params![key], |row| {
            row.get(0)
        })
        .optional()
        .context("get sync meta")
    }

    pub fn set_account_sync_status(
        &self,
        account_id: &str,
        status: &crate::models::AccountSyncStatus,
    ) -> Result<()> {
        let key = format!("account_sync:{account_id}");
        self.set_sync_meta(&key, &serde_json::to_string(status)?)
    }

    pub fn get_account_sync_status(
        &self,
        account_id: &str,
    ) -> Result<Option<crate::models::AccountSyncStatus>> {
        let key = format!("account_sync:{account_id}");
        Ok(self
            .get_sync_meta(&key)?
            .and_then(|json| serde_json::from_str(&json).ok()))
    }

    pub fn list_account_sync_statuses(&self) -> Result<Vec<crate::models::AccountSyncStatus>> {
        let accounts = self.list_accounts()?;
        let mut statuses = Vec::new();
        for account in accounts {
            if let Some(status) = self.get_account_sync_status(&account.id)? {
                statuses.push(status);
            } else {
                statuses.push(crate::models::AccountSyncStatus {
                    account_id: account.id,
                    display_name: account.display_name,
                    source: account.source,
                    last_sync: None,
                    last_error: None,
                    reminders_synced: 0,
                });
            }
        }
        Ok(statuses)
    }

    pub fn delete_account_sync_status(&self, account_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM sync_meta WHERE key = ?1",
            params![format!("account_sync:{account_id}")],
        )?;
        Ok(())
    }
}

fn parse_account_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarAccount> {
    let style_json: Option<String> = row.get(8)?;
    let style_overrides = style_json
        .and_then(|json| serde_json::from_str::<AccountStyleOverrides>(&json).ok());
    Ok(CalendarAccount {
        id: row.get(0)?,
        source: row.get(1)?,
        display_name: row.get(2)?,
        email: row.get(3)?,
        sync_token: row.get(4)?,
        caldav_url: row.get(5)?,
        caldav_username: row.get(6)?,
        connected_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| Utc::now()),
        style_overrides,
    })
}

fn parse_reminder_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReminderEvent> {
    Ok(ReminderEvent {
        id: row.get(0)?,
        account_id: row.get(1)?,
        source: row.get(2)?,
        external_id: row.get(3)?,
        title: row.get(4)?,
        start_time: row.get::<_, String>(5)?.parse().unwrap_or_else(|_| Utc::now()),
        reminder_time: row.get::<_, String>(6)?.parse().unwrap_or_else(|_| Utc::now()),
        location: row.get(7)?,
        url: row.get(8)?,
        fired_at: row
            .get::<_, Option<String>>(9)?
            .and_then(|s| s.parse().ok()),
        snoozed_until: row
            .get::<_, Option<String>>(10)?
            .and_then(|s| s.parse().ok()),
        dismissed: row.get::<_, i32>(11)? != 0,
    })
}

pub fn db_path() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .or_else(dirs::home_dir)
        .context("resolve data dir")?;
    Ok(dir.join("screen-reminder").join("screen-reminder.db"))
}

pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}
