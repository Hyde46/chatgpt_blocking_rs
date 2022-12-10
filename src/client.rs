use std::str::FromStr;

use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Url,
};

use crate::types::{PossiblyError, SessionRefresh};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClientOptions {
    api_url: Url,
    backend_api_url: Url,
    user_agent: String,
    markdown: bool,
}

impl ClientOptions {
    pub fn with_api_url(mut self, url: Url) -> Self {
        self.api_url = url;
        self
    }

    pub fn with_backend_api_url(mut self, backend_url: Url) -> Self {
        self.backend_api_url = backend_url;
        self
    }

    pub fn with_user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn with_markdown_support(mut self, markdown: bool) -> Self {
        self.markdown = markdown;
        self
    }
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            api_url: Url::from_str("https://chat.openai.com/api/").unwrap(),
            backend_api_url: Url::from_str("https://chat.openai.com/backend-api/").unwrap(),
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0 Safari/537.36".into(),
            markdown: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatGPT {
    client: reqwest::Client,
    options: ClientOptions,
    access_token: String,
}

impl ChatGPT {
    pub fn new<S: Into<String>>(token: S) -> crate::Result<Self> {
        Self::with_options(token, ClientOptions::default())
    }

    pub fn with_options<S: Into<String>>(token: S, options: ClientOptions) -> crate::Result<Self> {
        let token = token.into();
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_bytes(options.user_agent.as_bytes())?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .build()?;
        Ok(Self {
            client,
            options,
            access_token: token,
        })
    }

    pub async fn refresh_token(&mut self) -> crate::Result<String> {
        let refresh: PossiblyError<SessionRefresh> = self
            .client
            .get(
                self.options
                    .api_url
                    .join("auth/session")
                    .map_err(|err| crate::err::Error::ParsingError(err.to_string()))?,
            )
            .header(
                "Cookie",
                format!("__Secure-next-auth.session-token={}", self.access_token),
            )
            .send()
            .await?
            .json()
            .await?;
        match refresh {
            PossiblyError::Error { error } => Err(crate::err::Error::BackendError(error)),
            PossiblyError::Fine(refresh) => {
                self.access_token = refresh.access_token.clone();
                Ok(refresh.access_token)
            }
        }
    }
}