use crate::api::response::ResponseCreateEvent;
use crate::api::session::{Session, SessionUpdateEvent};
use crate::error::RealtimeError;
use crate::event::{Event, EventMessage};
use crate::websocket::config::WebsocketConfig;
use async_trait::async_trait;
use ezsockets::{Error, Utf8Bytes};
use nanoid::nanoid;
use serde::Serialize;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::{Mutex, oneshot};
use tracing::{debug, error, info};

pub mod config {
    use crate::ApiKeyRef;
    use crate::api::model::Model;
    use url::Url;

    #[derive(Debug)]
    pub struct WebsocketConfig {
        pub model: Model,
        pub api_key_ref: ApiKeyRef,
    }

    impl Default for WebsocketConfig {
        fn default() -> Self {
            Self {
                model: Model::default(),
                api_key_ref: ApiKeyRef::default(),
            }
        }
    }

    impl WebsocketConfig {
        pub fn url(&self) -> Url {
            Url::parse(format!("wss://api.openai.com/v1/realtime?model={}", self.model).as_str())
                .unwrap()
        }
    }
}

pub async fn connect(
    config: WebsocketConfig,
) -> Result<(Arc<RealtimeSession>, UnboundedReceiver<Vec<u8>>), RealtimeError> {
    let ws_config = ezsockets::ClientConfig::new(config.url())
        .bearer(config.api_key_ref.api_key())
        .header("openai-beta", "realtime=v1");

    let (tx_events, mut rx_events) = unbounded_channel();
    let (tx_connected, rx_connected) = oneshot::channel();

    let session_id = nanoid!(6);

    let (handle, _) = ezsockets::connect(
        |handle| WebsocketHandle {
            _handle: handle,
            session_id: session_id.clone(),
            tx_events,
            connected: Some(tx_connected),
        },
        ws_config,
    )
    .await;

    rx_connected.await.unwrap();

    info!("connected");

    // create new realtime session
    let (realtime_session, rx_audio) = RealtimeSession::new(session_id, Arc::new(handle));

    // process events
    let realtime_session_for_events = realtime_session.clone();
    tokio::spawn(async move {
        while let Some(evt) = rx_events.recv().await {
            realtime_session_for_events.handle_event(evt).await;
        }
    });

    Ok((realtime_session, rx_audio))
}

pub struct WebsocketHandle {
    _handle: ezsockets::Client<Self>,
    session_id: String,
    tx_events: UnboundedSender<Event>,
    connected: Option<oneshot::Sender<()>>,
}

#[async_trait]
impl ezsockets::ClientExt for WebsocketHandle {
    type Call = ();

    async fn on_text(&mut self, text: Utf8Bytes) -> Result<(), ezsockets::Error> {
        let j: Value = serde_json::from_str(text.as_str()).unwrap();

        let m = j.as_object().unwrap();
        let event_type = m.get("type").unwrap().as_str().unwrap();

        if event_type.to_string() != "response.audio.delta" {
            debug!(
                "openai: received event: {event_type}\n{}",
                serde_json::to_string_pretty(&j.clone()).unwrap()
            );
        }

        debug!("session({})> event: {}", self.session_id, event_type);

        match event_type {
            "session.created" => {
                self.tx_events
                    .send(Event::SessionCreated(
                        serde_json::from_value(m.get("session").unwrap().clone()).unwrap(),
                    ))
                    .unwrap();
            }
            "response.audio.delta" => {
                let decoded = base64::decode(m.get("delta").unwrap().as_str().unwrap()).unwrap();
                self.tx_events.send(Event::Audio(decoded)).unwrap();
            }
            "response.audio_transcript.delta" => {
                self.tx_events
                    .send(Event::TranscriptDelta(
                        serde_json::from_value(m.get("delta").unwrap().clone()).unwrap(),
                    ))
                    .unwrap();
            }
            "response.audio_transcript.done" => {
                self.tx_events
                    .send(Event::TranscriptDone(
                        serde_json::from_value(m.get("transcript").unwrap().clone()).unwrap(),
                    ))
                    .unwrap();
            }
            "input_audio_buffer.speech_started" => {
                self.tx_events
                    .send(Event::InputAudioBufferSpeechStarted)
                    .unwrap();
            }
            "response.audio.done" => {
                println!("response.audio.done {:?}", m);

                // TODO: when done, we should generate a bit of silence at then end

                self.tx_events.send(Event::AudioDone).unwrap();

                // TODO: figure out how much silence we actually need
                let silence: Vec<u8> = vec![0; 48_000 * 2];
                self.tx_events.send(Event::Audio(silence)).unwrap();
            }
            // TODO: response.audio.done
            // TODO: response.audio_transcript.done
            // TODO: response.done
            _ => debug!(
                "Unhandled event:\n{}",
                serde_json::to_string_pretty(&j.clone()).unwrap()
            ),
        }

        //tracing::debug!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, _bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        unimplemented!()
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), Error> {
        if let Some(connected) = self.connected.take() {
            connected.send(()).unwrap();
        }
        Ok(())
    }
}

pub struct RealtimeSession {
    id: String,
    session: Mutex<Option<Session>>,
    tx_audio: UnboundedSender<Vec<u8>>,
    tx_msg_out: UnboundedSender<Utf8Bytes>,
}

impl RealtimeSession {
    pub fn new(
        id: String,
        ws: Arc<ezsockets::Client<WebsocketHandle>>,
    ) -> (Arc<Self>, UnboundedReceiver<Vec<u8>>) {
        let (tx_audio_out, rx_audio_out) = unbounded_channel();

        let (tx_msg_out, mut rx_msg_out) = unbounded_channel::<Utf8Bytes>();

        let ws_2 = ws.clone();
        tokio::spawn(async move {
            while let Some(data) = rx_msg_out.recv().await {
                match ws_2.text(data) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("error sending: {}", e);
                    }
                }
            }
            panic!("websocket closed");
        });

        let session = Arc::new(Self {
            id,
            session: Mutex::new(None),
            tx_audio: tx_audio_out,
            tx_msg_out: tx_msg_out.clone(),
        });

        // TODO: send from websocket to tx_audio

        (session, rx_audio_out)
    }

    fn send(&self, evt: &str, body: impl Serialize) -> anyhow::Result<()> {
        let body_str = serde_json::to_string_pretty(&EventMessage::wrap(evt, body))?;
        if evt != "input_audio_buffer.append" {
            debug!("session({})> send: {} {}", self.id, evt, body_str);
        }
        self.tx_msg_out.send(Utf8Bytes::from(body_str))?;
        Ok(())
    }

    /// Updates the session
    /// See: https://platform.openai.com/docs/api-reference/realtime-client-events/session/update
    pub fn session_update(&self, session: SessionUpdateEvent) -> anyhow::Result<()> {
        self.send(
            "session.update",
            json!({
                "session": session
            }),
        )
    }

    /// This event instructs the server to create a Response, which means triggering model inference. When in Server VAD mode, the server will create Responses automatically.
    /// See: https://platform.openai.com/docs/api-reference/realtime-client-events/response/create
    pub fn response_create(&self, response: ResponseCreateEvent) -> anyhow::Result<()> {
        self.send(
            "response.create",
            json!({
                "response": response
            }),
        )
    }

    pub fn audio_append(&self, buffer: Vec<u8>) -> anyhow::Result<()> {
        debug!("session({})> audio --> {} bytes", self.id, buffer.len());
        self.send(
            "input_audio_buffer.append",
            json!({
                "audio": base64::encode(buffer)
            }),
        )
    }

    async fn handle_event(&self, evt: Event) {
        // debug
        match evt.clone() {
            Event::Audio(audio) => {
                debug!("session({})> audio <-- {} bytes", self.id, audio.len());
            }
            _ => debug!("{:?}", evt),
        }

        match evt {
            Event::Audio(audio) => match self.tx_audio.send(audio) {
                Ok(_) => {}
                Err(e) => {
                    error!("error handling audio event: {}", e);
                }
            },
            Event::SessionCreated(session) => {
                info!("Session created: {}", session.id);
                {
                    self.session.lock().await.replace(session);
                }
            }
            Event::TranscriptDone(transcript) => {
                info!("transcript done: {transcript}");
            }
            Event::InputAudioBufferSpeechStarted => {
                //println!("STARTED");
                //rt_client_events.send("response.cancel", json!({}))
                //rt_client_events.send("response.cancel", json!({}))
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::WebsocketConfig;
    use crate::websocket::connect;

    #[tokio::test]
    async fn it_works() {
        let client = connect(WebsocketConfig::default()).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }
}
