use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tiny_http::{Header, Response, Server};
use url::Url;

use crate::calendar::{reminder_from_parts, CalendarProvider, SyncResult};
use crate::calendar::google::GoogleProvider;
use crate::oauth_util::{open_browser, parse_query_param, pick_port};
use crate::models::CalendarAccount;
use crate::secrets;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_TASKS_URL: &str = "https://tasks.googleapis.com/tasks/v1/lists/@default/tasks";

pub struct GoogleTasksProvider {
    client: Client,
}

impl GoogleTasksProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    fn oauth_client(redirect_uri: &str) -> Result<BasicClient> {
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
        let (auth_url, _) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/tasks.readonly".to_string(),
            ))
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
            "<html><body><h1>Google Tasks connected!</h1><p>You can close this window.</p></body></html>",
        )
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
        let _ = request.respond(response);

        Ok(CalendarAccount {
            id: account_id,
            source: "google_tasks".to_string(),
            display_name: "Google Tasks".to_string(),
            email: None,
            sync_token: None,
            caldav_url: None,
            caldav_username: None,
            connected_at: Utc::now(),
            style_overrides: None,
        })
    }
}

impl CalendarProvider for GoogleTasksProvider {
    fn source(&self) -> &'static str {
        "google_tasks"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        tauri::async_runtime::block_on(self.sync_async(account))
    }
}

impl GoogleTasksProvider {
    async fn sync_async(&self, account: &CalendarAccount) -> Result<SyncResult> {
        let access_token = GoogleProvider::access_token_for_account(account).await?;
        let mut url = Url::parse(GOOGLE_TASKS_URL)?;
        url.query_pairs_mut()
            .append_pair("showCompleted", "false")
            .append_pair("showHidden", "false")
            .append_pair("maxResults", "100");

        let response = self
            .client
            .get(url)
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<GoogleTasksResponse>()
            .await?;

        let mut reminders = Vec::new();
        for task in response.items.unwrap_or_default() {
            let Some(due_raw) = task.due else {
                continue;
            };
            let due = due_raw
                .parse::<chrono::DateTime<Utc>>()
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(&due_raw, "%Y-%m-%d")
                        .map(|date| date.and_hms_opt(9, 0, 0).unwrap())
                        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
                })
                .unwrap_or_else(|_| Utc::now());

            let title = task.title.unwrap_or_else(|| "Untitled task".to_string());
            let id = task.id.unwrap_or_else(|| title.clone());

            reminders.push(reminder_from_parts(
                account,
                &id,
                &title,
                due,
                0,
                None,
                task.links.as_ref().and_then(|l| l.first()).and_then(|l| l.link.clone()),
            ));
        }

        Ok(SyncResult {
            reminders,
            sync_token: None,
        })
    }
}

#[derive(Deserialize)]
struct GoogleTasksResponse {
    items: Option<Vec<GoogleTask>>,
}

#[derive(Deserialize)]
struct GoogleTask {
    id: Option<String>,
    title: Option<String>,
    due: Option<String>,
    links: Option<Vec<GoogleTaskLink>>,
}

#[derive(Deserialize)]
struct GoogleTaskLink {
    link: Option<String>,
}
