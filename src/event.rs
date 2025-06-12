use crate::api::session::Session;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventMessage {
    pub event_id: String,

    #[serde(rename = "type")]
    pub event_type: String,

    #[serde(flatten)]
    pub body: serde_json::Value,
}

impl EventMessage {
    pub fn wrap(evt: &str, body: impl Serialize) -> Self {
        Self {
            event_id: nanoid!(),
            event_type: evt.to_string(),
            body: serde_json::to_value(body).unwrap(),
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Audio(Vec<u8>),
    SessionCreated(Session),
    TranscriptDelta(String),
    TranscriptDone(String),
    InputAudioBufferSpeechStarted,
    AudioDone,
}
