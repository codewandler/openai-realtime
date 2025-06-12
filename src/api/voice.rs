use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Voice {
    #[default]
    Alloy,
    Ash,
    Ballad,
    Coral,
    Echo,
    Sage,
    Shimmer,
    Verse,
}

impl From<&str> for Voice {
    fn from(s: &str) -> Self {
        serde_json::from_str(format!("\"{}\"", s).as_str()).unwrap()
    }
}

impl From<String> for Voice {
    fn from(s: String) -> Self {
        serde_json::from_str(format!("\"{}\"", s).as_str()).unwrap()
    }
}
