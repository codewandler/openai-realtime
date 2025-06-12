use crate::api::model::Model;
use crate::api::session::{ClientSecret, CreateSessionRequest, Session};
use crate::api::voice::Voice;
use crate::{ApiKeyRef, RealtimeError};

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub api_key_ref: ApiKeyRef,
    pub model: Model,
    pub voice: Voice,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            model: Model::default(),
            api_key_ref: ApiKeyRef::default(),
            voice: Voice::Verse,
        }
    }
}

/// Creates an ephemeral token for WebRTC
/// See: https://platform.openai.com/docs/guides/realtime#connect-with-webrtc
/// See: https://platform.openai.com/docs/guides/realtime#connection-details
pub async fn create_ephemeral_token(config: &SessionConfig) -> Result<ClientSecret, RealtimeError> {
    Ok(create_session(config).await?.client_secret.unwrap())
}

/// Create a new Session
pub async fn create_session(config: &SessionConfig) -> Result<Session, RealtimeError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/realtime/sessions")
        .header(
            "Authorization",
            format!("Bearer {}", config.api_key_ref.clone().api_key()),
        )
        .header("Content-Type", "application/json")
        .json(&CreateSessionRequest {
            model: config.model.clone(),
            voice: config.voice.clone(),
        })
        .send()
        .await
        .map_err(RealtimeError::Http)?
        .json()
        .await
        .map_err(RealtimeError::Http)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use crate::session::{SessionConfig, create_ephemeral_token};

    #[tokio::test]
    async fn test_get_token() {
        let token = create_ephemeral_token(&SessionConfig::default())
            .await
            .unwrap();
        println!("token: {token:?}");
    }
}
