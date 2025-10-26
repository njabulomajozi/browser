//! Browser networking layer
//!
//! HTTP/HTTPS client with caching and DNS resolution.

use anyhow::Result;
use url::Url;

/// HTTP client for fetching web resources
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("BrowserMVP/0.1.0")
            .build()?;

        Ok(Self { client })
    }

    /// Fetch a URL and return the response body
    pub async fn fetch(&self, url: Url) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let body = response.text().await?;
        Ok(body)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = HttpClient::new().unwrap();
    }

    #[tokio::test]
    async fn test_fetch_example_com() {
        let client = HttpClient::new().unwrap();
        let url = Url::parse("http://example.com").unwrap();
        let body = client.fetch(url).await.unwrap();
        assert!(body.contains("Example Domain"));
    }
}
