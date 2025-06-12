use crate::api::session::Modality;
use crate::api::voice::Voice;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ResponseCreateEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<Modality>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    // TODO: other inference options
}
