use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;

use crate::calendar::build_provider;
use crate::dedupe::dedupe_reminders;
use crate::models::{AccountSyncStatus, ReminderEvent};
use crate::storage::Storage;

pub struct SyncEngine {
    storage: Arc<Storage>,
}

pub struct SyncReport {
    pub reminders_upserted: usize,
    pub account_statuses: Vec<AccountSyncStatus>,
}

impl SyncEngine {
    pub fn new(storage: Arc<Storage>) -> Self {
        Self { storage }
    }

    pub async fn sync_all(&self) -> Result<SyncReport> {
        let settings = self.storage.get_settings()?;
        let accounts = self.storage.list_accounts()?;
        let mut all_reminders: Vec<ReminderEvent> = Vec::new();
        let mut account_statuses = Vec::new();

        for account in accounts {
            let mut status = AccountSyncStatus {
                account_id: account.id.clone(),
                display_name: account.display_name.clone(),
                source: account.source.clone(),
                last_sync: None,
                last_error: None,
                reminders_synced: 0,
            };

            let provider = match build_provider(&account.source) {
                Ok(provider) => provider,
                Err(err) => {
                    status.last_error = Some(err.to_string());
                    self.storage
                        .set_account_sync_status(&account.id, &status)?;
                    account_statuses.push(status);
                    continue;
                }
            };

            match provider.sync(&account) {
                Ok(result) => {
                    status.last_sync = Some(Utc::now());
                    status.reminders_synced = result.reminders.len();
                    all_reminders.extend(result.reminders);
                    if let Some(token) = result.sync_token {
                        let mut updated = account.clone();
                        updated.sync_token = Some(token);
                        self.storage.upsert_account(&updated)?;
                    }
                }
                Err(err) => {
                    status.last_error = Some(err.to_string());
                    log::error!(
                        "sync {} ({}) failed: {err}",
                        account.display_name,
                        account.source
                    );
                }
            }

            self.storage
                .set_account_sync_status(&account.id, &status)?;
            account_statuses.push(status);
        }

        if settings.dedupe_reminders {
            all_reminders = dedupe_reminders(all_reminders);
        }

        let reminders_upserted = if all_reminders.is_empty() {
            0
        } else {
            self.storage.upsert_reminders(&all_reminders)?
        };

        self.storage
            .set_sync_meta("last_sync", &Utc::now().to_rfc3339())?;

        Ok(SyncReport {
            reminders_upserted,
            account_statuses,
        })
    }
}
