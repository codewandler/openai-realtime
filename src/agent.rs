use crate::api::model::Model;
use crate::{
    AudioFormat, Modality, SessionUpdateEvent, TurnDetection, Voice, WebsocketConfig, websocket,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug, Clone, Default)]
pub struct AgentConfig {
    pub model: Option<Model>,
    pub voice: Option<Voice>,
    pub speed: Option<f32>,
    pub instructions: Option<String>,
}

pub async fn connect_realtime_agent(
    config: AgentConfig,
) -> anyhow::Result<(Arc<websocket::RealtimeSession>, UnboundedReceiver<Vec<u8>>)> {
    let voice = config.voice.unwrap_or(Voice::Echo);
    let model = config.model.unwrap_or(Model::default());

    // create a new realtime agent
    let rt_config = WebsocketConfig {
        model,
        ..Default::default()
    };
    if rt_config.api_key_ref.api_key().is_empty() {
        Err(anyhow::anyhow!(
            "invalid api key ref: {}",
            rt_config.api_key_ref
        ))?;
    }

    let (rt_client, rx_audio) = websocket::connect(rt_config).await.unwrap();

    let instructions = config.instructions.unwrap_or(
        r###"
You are Melissa, a helpful customer support agent.
You language is en-US.
"###
        .to_string(),
    );

    rt_client.session_update(SessionUpdateEvent {
        temperature: 0.7.into(),
        instructions: instructions.into(),
        speed: config.speed,
        voice: voice.clone().into(),
        modalities: vec![Modality::Audio, Modality::Text].into(),
        turn_detection: TurnDetection {
            create_response: true,
            interrupt_response: false,
            prefix_padding_ms: 300,
            silence_duration_ms: 1000,
            td_type: "server_vad".into(),
            threshold: 0.5,
        }
        .into(),
        input_audio_format: Some(AudioFormat::PCM16),
        output_audio_format: Some(AudioFormat::PCM16),
        ..Default::default()
    })?;

    Ok((rt_client, rx_audio))
}
