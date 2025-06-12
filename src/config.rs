use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum ApiKeyRef {
    Value(String),
    Env(Option<String>),
}

impl Display for ApiKeyRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyRef::Value(key) => write!(f, "{}", key),
            ApiKeyRef::Env(env) => write!(f, "{}", env.as_deref().unwrap_or("OPENAI_KEY")),
        }
    }
}

impl ApiKeyRef {
    /// Resolve the Api-Key
    pub fn api_key(&self) -> String {
        match &self {
            ApiKeyRef::Value(key) => key.to_string(),
            ApiKeyRef::Env(env) => {
                let env_key = env.clone().unwrap_or("OPENAI_KEY".to_string());
                std::env::var(env_key).unwrap_or_else(|_| "".to_string())
            }
        }
    }
}

impl Default for ApiKeyRef {
    fn default() -> Self {
        Self::Env("OPENAI_KEY".to_string().into())
    }
}
