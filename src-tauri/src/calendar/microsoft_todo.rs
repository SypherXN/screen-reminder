use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tiny_http::{Header, Response, Server};

use crate::config;
use crate::calendar::{parse_ical_datetime, parse_all_day_date, reminder_from_parts, CalendarProvider, SyncResult};
use crate::calendar::outlook::OutlookProvider;
use crate::oauth_util::{oauth_http_client, OAuthClient, MICROSOFT_OAUTH_PROMPT, parse_oauth_error};
use crate::models::CalendarAccount;
use crate::secrets;

const MS_AUTH_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
const MS_TODO_LISTS: &str = "https://graph.microsoft.com/v1.0/me/todo/lists";

pub struct MicrosoftTodoProvider {
    client: Client,
}

impl MicrosoftTodoProvider {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    fn oauth_client(redirect_uri: &str) -> Result<OAuthClient> {
        let client_id = config::require_env("MICROSOFT_CLIENT_ID")?;
        let client_secret = config::require_env("MICROSOFT_CLIENT_SECRET")?;

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
            .add_scope(Scope::new("Tasks.Read".to_string()))
            .add_scope(Scope::new("offline_access".to_string()))
            .add_scope(Scope::new("User.Read".to_string()))
            .add_extra_param("prompt", MICROSOFT_OAUTH_PROMPT)
            .set_pkce_challenge(pkce_challenge)
            .url();

        crate::oauth_util::open_browser(&auth_url.to_string())?;

        let server = Server::http(format!("127.0.0.1:{port}"))
            .map_err(|err| anyhow::anyhow!("oauth server: {err}"))?;
        let request = server.recv().map_err(|err| anyhow::anyhow!("oauth recv: {err}"))?;
        let query = request.url().split('?').nth(1).unwrap_or("");
        if let Some(error) = parse_oauth_error(query) {
            anyhow::bail!("Microsoft sign-in failed: {error}");
        }
        let code = crate::oauth_util::parse_query_param(query, "code")
            .context("missing authorization code")?;

        let http = oauth_http_client();
        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&http)
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
            "<html><body><h1>Microsoft To Do connected!</h1><p>You can close this window.</p></body></html>",
        )
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
        let _ = request.respond(response);

        Ok(CalendarAccount {
            id: account_id,
            source: "microsoft_todo".to_string(),
            display_name: "Microsoft To Do".to_string(),
            email: None,
            sync_token: None,
            caldav_url: None,
            caldav_username: None,
            connected_at: Utc::now(),
            style_overrides: None,
        })
    }
}

impl CalendarProvider for MicrosoftTodoProvider {
    fn source(&self) -> &'static str {
        "microsoft_todo"
    }

    fn sync(&self, account: &CalendarAccount) -> Result<SyncResult> {
        tauri::async_runtime::block_on(self.sync_async(account))
    }
}

impl MicrosoftTodoProvider {
    async fn sync_async(&self, account: &CalendarAccount) -> Result<SyncResult> {
        let access_token = OutlookProvider::access_token_for_account(account).await?;
        let lists = self
            .client
            .get(MS_TODO_LISTS)
            .bearer_auth(&access_token)
            .send()
            .await?
            .error_for_status()?
            .json::<TodoListsResponse>()
            .await?;

        let mut reminders = Vec::new();
        for list in lists.value.unwrap_or_default() {
            let Some(list_id) = list.id else {
                continue;
            };
            let url = format!("{MS_TODO_LISTS}/{list_id}/tasks?$filter=status ne 'completed'&$top=100");
            let tasks = self
                .client
                .get(url)
                .bearer_auth(&access_token)
                .send()
                .await?
                .error_for_status()?
                .json::<TodoTasksResponse>()
                .await?;

            for task in tasks.value.unwrap_or_default() {
                let Some(due_raw) = task.due_date_time.and_then(|d| d.date_time) else {
                    continue;
                };
                let Some(due_time) = parse_ical_datetime(&due_raw)
                    .ok()
                    .or_else(|| parse_all_day_date(&due_raw).ok())
                else {
                    log::warn!("skipping todo {:?}: could not parse due time", task.title);
                    continue;
                };
                let title = task.title.unwrap_or_else(|| "Untitled task".to_string());
                let id = task.id.unwrap_or_else(|| title.clone());

                reminders.push(reminder_from_parts(
                    account,
                    &id,
                    &title,
                    due_time,
                    0,
                    None,
                    task.web_url,
                ));
            }
        }

        Ok(SyncResult {
            reminders,
            sync_token: None,
        })
    }
}

#[derive(Deserialize)]
struct TodoListsResponse {
    value: Option<Vec<TodoList>>,
}

#[derive(Deserialize)]
struct TodoList {
    id: Option<String>,
}

#[derive(Deserialize)]
struct TodoTasksResponse {
    value: Option<Vec<TodoTask>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TodoTask {
    id: Option<String>,
    title: Option<String>,
    due_date_time: Option<TodoDateTime>,
    web_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TodoDateTime {
    date_time: Option<String>,
}
