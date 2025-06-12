use codewandler_audio::{AudioPlayback, convert_pcm16_bytes_to_f32};
use crossbeam_channel::Sender;
use openai_realtime::{
    AgentConfig, Model, RealtimeSession, ResponseCreateEvent, Voice, connect_realtime_agent,
};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = "en-US")]
    lang: String,

    #[arg(long, default_value = "1.2")]
    speed: Option<f32>,

    #[arg(long)]
    prompt1: Option<String>,

    /// Second prompt
    #[arg(long)]
    prompt2: Option<String>,

    #[arg(long)]
    starter: Option<String>,
}

fn pipe(
    playback: Sender<f32>,
    mut rx: UnboundedReceiver<Vec<u8>>,
    session: Arc<RealtimeSession>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(data) = rx.recv().await {
            for s in convert_pcm16_bytes_to_f32(data.clone()) {
                playback.send(s).unwrap()
            }

            session.audio_append(data).unwrap();
        }
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let sr = 24_000;
    let model = Model::default();

    let args = Args::parse();

    let (s1, s1_rx) = connect_realtime_agent(AgentConfig {
        instructions: args.prompt1.unwrap_or("You are Jen, your hobbies are programming and pizza. You try to find out something interesting about your conversation partner".to_string())
            .add(format!(". Your language is {}", args.lang).as_str())
            .into(),
        voice: Voice::Verse.into(),
        model: model.clone().into(),
        speed: args.speed.clone(),
        ..Default::default()
    }).await?;

    let (s2, s2_rx) = connect_realtime_agent(AgentConfig {
        instructions: args.prompt2.unwrap_or("You are Tobi, your hobbies are rock festivals and beer. You try to find out something interesting about your conversation partner".to_string())
            .add(format!(". Your language is {}", args.lang).as_str())
            .into(),
        voice: Voice::Sage.into(),
        model: model.clone().into(),
        speed: args.speed.clone(),
        ..Default::default()
    }).await?;

    s1.response_create(ResponseCreateEvent {
        instructions: args
            .starter
            .unwrap_or("You are a conversation partner.".to_string())
            .add(format!(". Your language is {}", args.lang).as_str())
            .into(),
        ..Default::default()
    })?;

    let pb = AudioPlayback::new(sr)?;
    let o1 = pb.new_output(sr);
    let o2 = pb.new_output(sr);

    let (r1, r2) = tokio::join!(pipe(o1, s1_rx, s2.clone()), pipe(o2, s2_rx, s1.clone()));
    r1.expect("pipe#1 failed");
    r2.expect("pipe#2 failed");

    Ok(())
}
