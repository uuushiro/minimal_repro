use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, RequestBuilder,
};
use std::env;

#[derive(Debug, Clone)]
pub struct DataHubClient {
    client: reqwest::Client,
    endpoint_url: String,
}

impl DataHubClient {
    pub fn new(endpoint_url: String) -> Self {
        let client = Self::create_client();
        Self {
            client,
            endpoint_url,
        }
    }

    pub fn query_executer(&self, token: String) -> RequestBuilder {
        let headers = self.create_headers(token);
        self.client.post(&self.endpoint_url).headers(headers)
    }

    fn create_client() -> Client {
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
        Client::builder()
            .user_agent(user_agent)
            .build()
            .expect("Failed to create reqwest client")
    }

    fn create_headers(&self, token: String) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let token_value = HeaderValue::from_str(&token);
        match token_value {
            Ok(mut token_value) => {
                token_value.set_sensitive(true);
                headers.insert("IdToken", token_value);
                headers
            }
            Err(_) => headers,
        }
    }
}
