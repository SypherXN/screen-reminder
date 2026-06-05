use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::calendar::{reminder_from_parts, CalendarProvider, SyncResult};
use crate::oauth_util::{open_browser, parse_query_param, pick_port};
use crate::models::CalendarAccount;
use crate::secrets;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_EVENTS_URL: &str = "https://www.googleapis.com/calendar/v3/calendars/primary/events";

pub struct GoogleProvider {
    client: Client,
}

impl GoogleProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    pub fn oauth_client(redirect_uri: &str) -> Result<BasicClient> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").context("GOOGLE_CLIENT_ID not set")?;
        let client_secret =
            std::env::var("GOOGLE_CLIENT_SECRET").context("GOOGLE_CLIENT_SECRET not set")?;

        Ok(BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(AuthUrl::new(GOOGLE_AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(GOOGLE_TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?))
    }

    pub async fn connect() -> Result<CalendarAccount> {
        let port = pick_port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        let client = Self::oauth_client(&redirect_uri)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _csrf) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        open_browser(&auth_url.to_string())?;

        let server = Server::http(format!("127.0.0.1:{port}"))
            .map_err(|err| anyhow::anyhow!("oauth server: {err}"))?;

        let request = server
            .recv()
            .map_err(|err| anyhow::anyhow!("oauth recv: {err}"))?;
        let query = request.url().split('?').nth(1).unwrap_or("");
        let code = parse_query_param(query, "code").context("missing authorization code")?;

        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .context("exchange token")?;

        let access_token = token.access_token().secret().clone();
        let refresh_token = token
            .refresh_token()
            .map(|t| t.secret().clone())
            .context("missing refresh token")?;

        let email = fetch_user_email(&access_token).await.unwrap_or_default();
        let account_id = crate::storage::new_id();
        secrets::store_tokens(
            &account_id,
            &secrets::OAuthTokens {
                access_token,
                refresh_token: Some(refresh_token),
                expires_at: token
                    .expires_in()
                    .map(|d| Utc::now() + Duration::seconds(d.as_secs() as i64)),
            },
        )?;

        let response = Response::from_string(
            "<html><body><h1>Connected!</h1><p>You can close this window.</p></body></html>",
        )
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
        let _ = request.respond(response);

        Ok(CalendarAccount {
            id: account_id,
            source: "google".to_string(),
            display_name: if email.is_empty() {
                "Google Calendar".to_string()
            } else {
                format!("Google ({email})")
            },
            email: if email.is_empty() { None } else { Some(email) },
            sync_token: None,
            caldav_url: None,
            caldav_username: None,
            connected_at: Utc::now(),
            style_overrides: None,
        })
    }

    async fn access_token(&self, account: &CalendarAccount) -> Result<String> {
        let tokens = secrets::load_tokens(&account.id)?.context("missing google tokens")?;
        if let Some(expires_at) = tokens.expires_at {
            if expires_at > Utc::now() + Duration::seconds(60) {
                return Ok(tokens.access_token);
            }
        }

        let refresh = tokens
            .refresh_token
            .context("missing refresh token for google account")?;
        let client_id = std::env::var("GOOGLE_CLIENT_ID")?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;

        #[derive(Deserialize)]
        struct TokenResponseBody {
            access_token: String,
            expires_in: Option<i64>,
        }

        let response = self
            .client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("refresh_token", refresh.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<TokenResponseBody>()
            .await?;

        secrets::store_tokens(
            &account.id,
            &secrets::OAuthTokens {
                access_token: response.access_token.clone(),
                refresh_token: Some(refresh),
                expires_at: response
                    .expires_in
                    .map(|secs| Utc::now() + Duration::seconds(secs)),
            },
        )?;

        Ok(response.access_token)
    }

    pub async fn access_token_for_account(account: &CalendarAccount) -> Result<String> {
        Self::new()?.access_token(account).await
    }
}

impl CalendarProvider for GoogleProvider {
    fn source(&self) -> &'static str {
        "google"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        tauri::async_runtime::block_on(self.sync_async(account))
    }
}

impl GoogleProvider {
    async fn sync_async(&self, account: &CalendarAccount) -> Result<SyncResult> {
        let access_token = self.access_token(account).await?;
        let time_min = Utc::now();
        let time_max = Utc::now() + Duration::days(30);

        let mut url = Url::parse(GOOGLE_EVENTS_URL)?;
        url.query_pairs_mut()
            .append_pair("singleEvents", "true")
            .append_pair("showDeleted", "false")
            .append_pair("timeMin", &time_min.to_rfc3339())
            .append_pair("timeMax", &time_max.to_rfc3339())
            .append_pair("maxResults", "250");

        if let Some(token) = &account.sync_token {
            url.query_pairs_mut().append_pair("syncToken", token);
        }

        let response = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<GoogleEventsResponse>()
            .await?;

        let mut reminders = Vec::new();
        for item in response.items.unwrap_or_default() {
            if item.status.as_deref() == Some("cancelled") {
                continue;
            }
            let title = item.summary.unwrap_or_else(|| "Untitled event".to_string());
            let start = item
                .start
                .and_then(|s| s.date_time.or(s.date))
                .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok())
                .unwrap_or_else(Utc::now);

            let minutes_before = item
                .reminders
                .and_then(|r| r.overrides)
                .and_then(|overrides| overrides.first().map(|o| o.minutes))
                .unwrap_or(10);

            reminders.push(reminder_from_parts(
                account,
                &item.id.unwrap_or_else(|| title.clone()),
                &title,
                start,
                minutes_before as i64,
                item.location,
                item.html_link,
            ));
        }

        Ok(SyncResult {
            reminders,
            sync_token: response.next_sync_token,
        })
    }
}

#[derive(Deserialize)]
struct GoogleEventsResponse {
    items: Option<Vec<GoogleEvent>>,
    next_sync_token: Option<String>,
}

#[derive(Deserialize)]
struct GoogleEvent {
    id: Option<String>,
    summary: Option<String>,
    status: Option<String>,
    location: Option<String>,
    html_link: Option<String>,
    start: Option<GoogleEventTime>,
    reminders: Option<GoogleReminders>,
}

#[derive(Deserialize)]
struct GoogleEventTime {
    date_time: Option<String>,
    date: Option<String>,
}

#[derive(Deserialize)]
struct GoogleReminders {
    overrides: Option<Vec<GoogleReminderOverride>>,
}

#[derive(Deserialize)]
struct GoogleReminderOverride {
    minutes: i32,
}

async fn fetch_user_email(access_token: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct UserInfo {
        email: Option<String>,
    }

    let info = Client::new()
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<UserInfo>()
        .await?;
    Ok(info.email.unwrap_or_default())
}
