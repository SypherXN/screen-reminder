use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tiny_http::{Header, Response, Server};

use crate::calendar::{reminder_from_parts, CalendarProvider, SyncResult};
use crate::models::CalendarAccount;
use crate::secrets;

const MS_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MS_GRAPH_EVENTS: &str = "https://graph.microsoft.com/v1.0/me/calendarView";

pub struct OutlookProvider {
    client: Client,
}

impl OutlookProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    fn oauth_client(redirect_uri: &str) -> Result<BasicClient> {
        let client_id = std::env::var("MICROSOFT_CLIENT_ID").context("MICROSOFT_CLIENT_ID not set")?;
        let client_secret = std::env::var("MICROSOFT_CLIENT_SECRET")
            .context("MICROSOFT_CLIENT_SECRET not set")?;

        Ok(BasicClient::new(ClientId::new(client_id))
            .set_client_secret(ClientSecret::new(client_secret))
            .set_auth_uri(AuthUrl::new(MS_AUTH_URL.to_string())?)
            .set_token_uri(TokenUrl::new(MS_TOKEN_URL.to_string())?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?))
    }

    pub async fn connect() -> Result<CalendarAccount> {
        let port = crate::oauth_util::pick_port();
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        let client = Self::oauth_client(&redirect_uri)?;

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("Calendars.Read".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .add_scope(Scope::new("User.Read".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        crate::oauth_util::open_browser(&auth_url.to_string())?;

        let server = Server::http(format!("127.0.0.1:{port}"))
            .map_err(|err| anyhow::anyhow!("oauth server: {err}"))?;
        let request = server.recv().map_err(|err| anyhow::anyhow!("oauth recv: {err}"))?;
        let query = request.url().split('?').nth(1).unwrap_or("");
        let code = crate::oauth_util::parse_query_param(query, "code")
            .context("missing authorization code")?;

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

        let profile = fetch_profile(&access_token).await?;
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
            source: "outlook".to_string(),
            display_name: profile
                .display_name
                .clone()
                .unwrap_or_else(|| "Outlook Calendar".to_string()),
            email: profile.mail.or(profile.user_principal_name),
            sync_token: None,
            caldav_url: None,
            caldav_username: None,
            connected_at: Utc::now(),
            style_overrides: None,
        })
    }

    async fn access_token(&self, account: &CalendarAccount) -> Result<String> {
        let tokens = secrets::load_tokens(&account.id)?.context("missing outlook tokens")?;
        if let Some(expires_at) = tokens.expires_at {
            if expires_at > Utc::now() + Duration::seconds(60) {
                return Ok(tokens.access_token);
            }
        }

        let refresh = tokens
            .refresh_token
            .context("missing refresh token for outlook account")?;
        let client_id = std::env::var("MICROSOFT_CLIENT_ID")?;
        let client_secret = std::env::var("MICROSOFT_CLIENT_SECRET")?;

        #[derive(Deserialize)]
        struct TokenResponseBody {
            access_token: String,
            expires_in: Option<i64>,
        }

        let response = self
            .client
            .post(MS_TOKEN_URL)
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

impl CalendarProvider for OutlookProvider {
    fn source(&self) -> &'static str {
        "outlook"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        tauri::async_runtime::block_on(self.sync_async(account))
    }
}

impl OutlookProvider {
    async fn sync_async(&self, account: &CalendarAccount) -> Result<SyncResult> {
        let access_token = self.access_token(account).await?;
        let start = Utc::now();
        let end = Utc::now() + Duration::days(30);

        let url = format!(
            "{MS_GRAPH_EVENTS}?startDateTime={}&endDateTime={}&$select=id,subject,start,end,location,isReminderOn,reminderMinutesBeforeStart,webLink&$top=250",
            urlencoding::encode(&start.to_rfc3339()),
            urlencoding::encode(&end.to_rfc3339())
        );

        let response = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .header("Prefer", "outlook.timezone=\"UTC\"")
            .send()
            .await?
            .error_for_status()?
            .json::<GraphEventsResponse>()
            .await?;

        let mut reminders = Vec::new();
        for event in response.value.unwrap_or_default() {
            if event.is_reminder_on == Some(false) {
                continue;
            }
            let minutes = event.reminder_minutes_before_start.unwrap_or(15) as i64;
            let start_time = event
                .start
                .and_then(|s| s.date_time)
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(Utc::now);

            reminders.push(reminder_from_parts(
                account,
                &event.id.unwrap_or_else(|| "unknown".to_string()),
                &event.subject.unwrap_or_else(|| "Untitled event".to_string()),
                start_time,
                minutes,
                event.location.and_then(|l| l.display_name),
                event.web_link,
            ));
        }

        Ok(SyncResult {
            reminders,
            sync_token: None,
        })
    }
}

#[derive(Deserialize)]
struct GraphEventsResponse {
    value: Option<Vec<GraphEvent>>,
}

#[derive(Deserialize)]
struct GraphEvent {
    id: Option<String>,
    subject: Option<String>,
    start: Option<GraphDateTime>,
    is_reminder_on: Option<bool>,
    reminder_minutes_before_start: Option<i32>,
    location: Option<GraphLocation>,
    web_link: Option<String>,
}

#[derive(Deserialize)]
struct GraphDateTime {
    date_time: Option<String>,
}

#[derive(Deserialize)]
struct GraphLocation {
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct GraphProfile {
    display_name: Option<String>,
    mail: Option<String>,
    user_principal_name: Option<String>,
}

async fn fetch_profile(access_token: &str) -> Result<GraphProfile> {
    Ok(Client::new()
        .get("https://graph.microsoft.com/v1.0/me")
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<GraphProfile>()
        .await?)
}
