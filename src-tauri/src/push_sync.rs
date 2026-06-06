use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config;
use crate::models::{CalendarAccount, PushSubscription};
use crate::storage::Storage;

const GOOGLE_WATCH_URL: &str =
    "https://www.googleapis.com/calendar/v3/calendars/primary/events/watch";
const MS_SUBSCRIPTIONS_URL: &str = "https://graph.microsoft.com/v1.0/subscriptions";

#[derive(Debug, Deserialize)]
struct RelayNotification {
    account_id: String,
    source: Option<String>,
}

pub fn device_id(storage: &Storage) -> Result<String> {
    if let Some(id) = storage.get_sync_meta("push_device_id")? {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    storage.set_sync_meta("push_device_id", &id)?;
    Ok(id)
}

pub fn relay_url() -> Option<String> {
    config::env_var("PUSH_RELAY_URL")
}

pub async fn register_all(storage: Arc<Storage>) -> Result<()> {
    let settings = storage.get_settings()?;
    if !settings.push_sync_enabled {
        return Ok(());
    }

    let Some(relay) = relay_url() else {
        return Ok(());
    };

    let device_id = device_id(&storage)?;
    let accounts = storage.list_accounts()?;
    let client = Client::new();

    for account in accounts {
        if should_skip_push(&account.source) {
            continue;
        }

        if let Ok(sub) = register_account_push(&client, &relay, &device_id, &account).await {
            storage.save_push_subscription(&sub)?;
        }
    }

    Ok(())
}

fn should_skip_push(source: &str) -> bool {
    matches!(source, "caldav" | "apple" | "google_tasks" | "microsoft_todo")
}

async fn register_account_push(
    client: &Client,
    relay: &str,
    device_id: &str,
    account: &CalendarAccount,
) -> Result<PushSubscription> {
    match account.source.as_str() {
        "google" => register_google_watch(client, relay, device_id, account).await,
        "outlook" => register_outlook_subscription(client, relay, device_id, account).await,
        _ => anyhow::bail!("unsupported push source: {}", account.source),
    }
}

async fn register_google_watch(
    client: &Client,
    relay: &str,
    device_id: &str,
    account: &CalendarAccount,
) -> Result<PushSubscription> {
    let access_token = google_access_token(account).await?;
    let channel_id = Uuid::new_v4().to_string();
    let address = format!("{relay}/webhook/google");
    let token = format!("{device_id}:{account_id}", account_id = account.id);

    #[derive(Serialize)]
    struct WatchRequest<'a> {
        id: &'a str,
        #[serde(rename = "type")]
        kind: &'a str,
        address: &'a str,
        token: &'a str,
    }

    let response = client
        .post(GOOGLE_WATCH_URL)
        .bearer_auth(access_token)
        .json(&WatchRequest {
            id: &channel_id,
            kind: "web_hook",
            address: &address,
            token: &token,
        })
        .send()
        .await
        .context("google watch request")?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("google watch failed: {body}");
    }

    #[derive(Deserialize)]
    struct WatchResponse {
        resourceId: String,
        expiration: String,
    }

    let body: WatchResponse = response.json().await.context("parse google watch")?;
    let expiration = body
        .expiration
        .parse::<i64>()
        .map(|ms| DateTime::<Utc>::from_timestamp_millis(ms).unwrap_or_else(Utc::now))
        .unwrap_or_else(|_| Utc::now() + Duration::days(7));

    Ok(PushSubscription {
        account_id: account.id.clone(),
        source: account.source.clone(),
        channel_id,
        resource_id: Some(body.resourceId),
        expiration,
    })
}

async fn register_outlook_subscription(
    client: &Client,
    relay: &str,
    device_id: &str,
    account: &CalendarAccount,
) -> Result<PushSubscription> {
    let access_token = outlook_access_token(account).await?;
    let channel_id = Uuid::new_v4().to_string();
    let notification_url = format!("{relay}/webhook/outlook");
    let client_state = format!("{device_id}:{account_id}", account_id = account.id);
    let expiration = Utc::now() + Duration::minutes(4200);

    #[derive(Serialize)]
    struct SubscriptionRequest<'a> {
        changeType: &'a str,
        notificationUrl: &'a str,
        resource: &'a str,
        expirationDateTime: String,
        clientState: &'a str,
    }

    let response = client
        .post(MS_SUBSCRIPTIONS_URL)
        .bearer_auth(access_token)
        .json(&SubscriptionRequest {
            changeType: "created,updated,deleted",
            notificationUrl: &notification_url,
            resource: "/me/events",
            expirationDateTime: expiration.to_rfc3339(),
            clientState: &client_state,
        })
        .send()
        .await
        .context("outlook subscription request")?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("outlook subscription failed: {body}");
    }

    #[derive(Deserialize)]
    struct SubscriptionResponse {
        id: String,
    }

    let body: SubscriptionResponse = response
        .json()
        .await
        .context("parse outlook subscription")?;

    Ok(PushSubscription {
        account_id: account.id.clone(),
        source: account.source.clone(),
        channel_id: body.id,
        resource_id: None,
        expiration,
    })
}

async fn google_access_token(account: &CalendarAccount) -> Result<String> {
    crate::calendar::google::GoogleProvider::access_token_for_account(account).await
}

async fn outlook_access_token(account: &CalendarAccount) -> Result<String> {
    crate::calendar::outlook::OutlookProvider::access_token_for_account(account).await
}

pub async fn poll_relay(storage: Arc<Storage>) -> Result<Vec<RelayNotification>> {
    let settings = storage.get_settings()?;
    if !settings.push_sync_enabled {
        return Ok(vec![]);
    }

    let Some(relay) = relay_url() else {
        return Ok(vec![]);
    };

    let device_id = device_id(&storage)?;
    let url = format!("{relay}/poll/{device_id}");
    let client = Client::new();
    let response = client.get(&url).send().await;

    let Ok(response) = response else {
        return Ok(vec![]);
    };

    if !response.status().is_success() {
        return Ok(vec![]);
    }

    Ok(response.json().await.unwrap_or_default())
}

pub async fn renew_expiring(storage: Arc<Storage>) -> Result<()> {
    let subs = storage.list_push_subscriptions()?;
    let now = Utc::now();
    for sub in subs {
        if sub.expiration <= now + Duration::hours(24) {
            let _ = register_all(storage.clone()).await;
            break;
        }
    }
    Ok(())
}

pub fn spawn_push_poll_loop(storage: Arc<Storage>, on_notify: impl Fn() + Send + Sync + 'static) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;

            if relay_url().is_none() {
                continue;
            }

            if let Err(err) = renew_expiring(storage.clone()).await {
                log::debug!("push renew check: {err}");
            }

            match poll_relay(storage.clone()).await {
                Ok(notifications) if !notifications.is_empty() => {
                    log::info!("push relay: {} notification(s)", notifications.len());
                    on_notify();
                }
                Ok(_) => {}
                Err(err) => log::debug!("push poll: {err}"),
            }
        }
    });
}
