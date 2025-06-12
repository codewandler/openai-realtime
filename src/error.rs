#[derive(Debug)]
pub enum RealtimeError {
    Serialization(serde_json::Error),
    Http(reqwest::Error),
    Websocket(ezsockets::Error),
}
