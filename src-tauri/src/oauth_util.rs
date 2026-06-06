use oauth2::basic::BasicClient;
use oauth2::{EndpointNotSet, EndpointSet};
use rand::Rng;

pub type OAuthClient = BasicClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub fn oauth_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build oauth HTTP client")
}

pub fn pick_port() -> u16 {
    rand::thread_rng().gen_range(49152..65535)
}

/// Force the Google account picker so users can connect multiple Google accounts.
pub const GOOGLE_OAUTH_PROMPT: &str = "select_account consent";

/// Force the Microsoft account picker for multiple Outlook / To Do accounts.
pub const MICROSOFT_OAUTH_PROMPT: &str = "select_account";

pub fn parse_query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        let v = parts.next()?;
        if k == key {
            Some(urlencoding::decode(v).ok()?.into_owned())
        } else {
            None
        }
    })
}

pub fn parse_oauth_error(query: &str) -> Option<String> {
    let error = parse_query_param(query, "error")?;
    let description = parse_query_param(query, "error_description");
    Some(description.unwrap_or(error))
}

pub fn open_browser(url: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // `cmd /C start` re-parses the command line and treats `&` in OAuth URLs as
        // command separators, truncating the URL and dropping params like `scope`.
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}
