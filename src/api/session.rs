use crate::api::model::Model;
use crate::api::voice::Voice;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub model: Model,
    pub voice: Voice,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnDetection {
    #[serde(rename = "type")]
    pub td_type: String,

    pub threshold: f32,

    pub prefix_padding_ms: i64,
    pub silence_duration_ms: i64,
    pub create_response: bool,
    pub interrupt_response: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    PCM16,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Audio,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    Auto,
    // TODO:
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientSecret {
    pub value: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    // TODO: !
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tracing {
    Auto,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SessionUpdateEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,

    ///  To clear a field like instructions, pass an empty string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<AudioFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<Tracing>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: String,

    pub object: String,

    pub expires_at: i64,

    // TODO: !
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_noise_reduction: Option<Value>,

    pub turn_detection: TurnDetection,

    pub input_audio_format: AudioFormat,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<Value>,

    pub include: Value,

    pub model: String,

    pub modalities: Vec<Modality>,

    pub instructions: String,

    pub voice: Voice,

    pub output_audio_format: AudioFormat,

    pub tool_choice: ToolChoice,

    pub temperature: f32,

    pub max_response_output_tokens: Value,

    pub speed: f32,

    pub tracing: Value,

    pub tools: Tool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<ClientSecret>,
}
