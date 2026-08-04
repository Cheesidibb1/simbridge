// REST API client utilities

use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// REST API client
pub struct RestClient {
    base_url: String,
    client: Client,
    auth_token: Option<String>,
}

impl RestClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
            auth_token: None,
        }
    }

    pub fn with_auth_token(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let mut builder = self.client.request(method, &url);
        
        if let Some(token) = &self.auth_token {
            builder = builder.header("Authorization", format!("Bearer {}", token));
        }
        
        builder
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, RestError> {
        let response = self.request(Method::GET, path)
            .send()
            .await
            .map_err(|e| RestError::RequestError(e.to_string()))?;
        
        self.handle_response(response).await
    }

    pub async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(&self, path: &str, body: &B) -> Result<T, RestError> {
        let response = self.request(Method::POST, path)
            .json(body)
            .send()
            .await
            .map_err(|e| RestError::RequestError(e.to_string()))?;
        
        self.handle_response(response).await
    }

    pub async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(&self, path: &str, body: &B) -> Result<T, RestError> {
        let response = self.request(Method::PUT, path)
            .json(body)
            .send()
            .await
            .map_err(|e| RestError::RequestError(e.to_string()))?;
        
        self.handle_response(response).await
    }

    pub async fn delete<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, RestError> {
        let response = self.request(Method::DELETE, path)
            .send()
            .await
            .map_err(|e| RestError::RequestError(e.to_string()))?;
        
        self.handle_response(response).await
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(&self, response: reqwest::Response) -> Result<T, RestError> {
        let status = response.status();
        
        if status.is_success() {
            response.json()
                .await
                .map_err(|e| RestError::DeserializationError(e.to_string()))
        } else {
            let error_text = response.text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(RestError::ApiError(status.as_u16(), error_text))
        }
    }
}

/// REST API errors
#[derive(Debug, Error)]
pub enum RestError {
    #[error("Request error: {0}")]
    RequestError(String),
    
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    
    #[error("API error (status {0}): {1}")]
    ApiError(u16, String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
