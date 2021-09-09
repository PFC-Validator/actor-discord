use crate::{NAME, VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GatewayReply {
    pub url: String,
}

pub const GATEWAY: usize = 0;
pub const HEARTBEAT: usize = 1;
pub const IDENTIFY: usize = 2;
pub const INVALID_SESSION: usize = 9;
pub const HELLO: usize = 10;
pub const ACK: usize = 11;

#[derive(Debug, Serialize, Deserialize)]
pub struct EventGuildCreate {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayHello {
    pub heartbeat_interval: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayIdentify {
    pub token: String,
    pub intents: u64,
    pub properties: HashMap<String, String>,
    pub v: usize,
}
impl GatewayIdentify {
    pub fn create(token: &str, intents: u64) -> Self {
        let mut p: HashMap<String, String> = Default::default();
        p.insert("$os".into(), "linux".into());
        p.insert("$browser".into(), NAME.unwrap_or("PFC-Discord").into());
        p.insert("$device".into(), VERSION.unwrap_or("DEV").into());

        GatewayIdentify {
            token: String::from(token),
            intents,
            properties: p,
            v: 9,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GatewayMessage {
    pub t: Option<String>,
    pub s: Option<usize>,
    pub op: usize,
    pub d: serde_json::Value,
}
