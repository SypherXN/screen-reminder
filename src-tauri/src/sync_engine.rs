use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;

use crate::calendar::build_provider;
use crate::models::AccountSyncStatus;
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
        let accounts = self.storage.list_accounts()?;
        let mut reminders_upserted = 0;
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

            if let Err(err) = build_provider(&account.source) {
                status.last_error = Some(err.to_string());
                self.storage
                    .set_account_sync_status(&account.id, &status)?;
                account_statuses.push(status);
                continue;
            }

            let source = account.source.clone();
            let account_for_sync = account.clone();

            // Provider sync uses async HTTP internally via block_on; run it off the
            // Tauri async runtime thread to avoid deadlocks/panics after OAuth.
            let sync_result = tokio::task::spawn_blocking(move || {
                let provider = build_provider(&source)?;
                provider.sync(&account_for_sync)
            })
            .await
            .context("calendar sync task failed")?;

            match sync_result {
                Ok(result) => {
                    status.last_sync = Some(Utc::now());
                    status.reminders_synced = result.reminders.len();
                    reminders_upserted += self.storage.replace_reminders_for_account(
                        &account.id,
                        &result.reminders,
                    )?;
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

        self.storage
            .set_sync_meta("last_sync", &Utc::now().to_rfc3339())?;

        Ok(SyncReport {
            reminders_upserted,
            account_statuses,
        })
    }
}
