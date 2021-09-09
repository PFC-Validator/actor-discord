use crate::errors::ActorDiscordError;
use crate::types::events::{
    ChannelEvent, Event, Guild, GuildChannel, GuildCreate, MessageEvent, MessageObject, SnowflakeID,
};
use crate::types::gateway::{GatewayHello, GatewayIdentify, GatewayMessage, GatewayReply};
use crate::{types::gateway, GatewayIntents};
//use crate::{NAME, VERSION};
use actix_http::ws::Frame;
use anyhow::Result;
use awc::http::StatusCode;
use awc::ws::Message;
//use awc::{ws, Client, ClientBuilder};
use awc::Client;
use futures::StreamExt;
use futures_util::sink::SinkExt as _;
//use futures_util::{sink::SinkExt as _, stream::StreamExt as _};
use actix_broker::{Broker, SystemBroker};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
//use tokio::sync::mpsc;
//use tokio::sync::watch;
use tokio::time::Interval;
use url::Url;

const API_PREFIX: &str = "/api/v9/";
const GATEWAY: &str = "gateway";
const GUILD_ID: &str = "guilds/";
pub struct DiscordAPI {
    pub client: Client,
    pub base_url: Url,
    pub token: String,
}
impl DiscordAPI {
    pub fn create(token: &str, connect_addr: &str) -> Result<DiscordAPI> {
        let base_url: Url = Url::from_str(connect_addr)?.join(API_PREFIX)?;
        let client = Client::builder().finish();
        Ok(DiscordAPI {
            client,
            base_url,
            token: token.into(),
        })
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, url_suffix: &str) -> anyhow::Result<T> {
        let full_url = self.base_url.join(url_suffix)?;
        log::info!("URL={}", full_url.as_str());
        let mut response = self
            .client
            .get(full_url.as_str())
            .insert_header(("User-Agent", "PFC-Discord"))
            .insert_header(("Authorization", format!("Bot {}", self.token)))
            .send()
            .await
            .map_err(|source| {
                eprintln!("{:#?}", source);
                ActorDiscordError::ResponseError()
            })?;
        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            Ok(response.json::<T>().limit(1024 * 1024).await?)
        } else {
            log::error!(
                "{} {}",
                response.status(),
                std::str::from_utf8(&response.body().limit(6000).await.unwrap())?
            );
            Err(ActorDiscordError::ResponseError().into())
        }
    }

    pub async fn guild(&self, id: SnowflakeID) -> Result<Guild> {
        let url = self.base_url.join(GUILD_ID)?.join(&id.to_string())?;
        let guild: Guild = self.get(url.as_str()).await?;
        Ok(guild)
    }
}
pub struct DiscordBot<'a> {
    pub api: &'a DiscordAPI,
    pub client: Client,
    pub web_socket: Url,
    pub intents: u64,
    pub duration: Duration,
    pub sequence_number: Option<usize>,
    pub interval: Interval,
}
impl<'a> DiscordBot<'a> {
    pub async fn create(api: &'a DiscordAPI, intents: GatewayIntents) -> Result<DiscordBot<'a>> {
        let mut config = rustls::ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let protos = vec![b"http/1.1".to_vec()];
        config.set_protocols(&protos);

        let rc_config = Arc::new(config);
        let client = Client::builder()
            .connector(awc::Connector::new().rustls(rc_config))
            .finish();

        //   let base_url: Url = Url::from_str(connect_addr)?.join(API_PREFIX)?;

        //        log::error!("URL_{}", base_url.as_str());
        let web_socket = api.get::<GatewayReply>(GATEWAY).await?;

        let web_socket_url = Url::from_str(&web_socket.url)?;
        let duration = Duration::from_secs(1);
        Ok(DiscordBot {
            client,
            api,
            web_socket: web_socket_url,
            //  tx,
            //  rx,
            intents: intents.bits,
            duration,
            sequence_number: None,
            interval: tokio::time::interval(duration),
        })
    }

    async fn handle_ws_gateway_event(
        &mut self,
        event_name: &str,
        gateway_message: serde_json::Value,
    ) -> Result<(bool, Option<Message>)> {
        match event_name {
            "GUILD_CREATE" => {
                let gc: GuildCreate = serde_json::from_value(gateway_message)?;
                let event = Event::GuildCreate(gc);
                log::debug!("Guild Create");
                Broker::<SystemBroker>::issue_async(event);
            }
            "READY" => {
                // TODO log session id for resumes
                log::info!("READY\n{}", gateway_message)
            }
            "MESSAGE_CREATE" | "MESSAGE_UPDATE" | "MESSAGE_DELETE" => {
                // log::info!("{}\n{}", event_name, gateway_message);
                let gc: MessageObject = serde_json::from_value(gateway_message)?;
                let event = if event_name == "MESSAGE_CREATE" {
                    MessageEvent::MessageCreate(gc)
                } else if event_name == "MESSAGE_DELETE" {
                    MessageEvent::MessageDelete(gc)
                } else {
                    MessageEvent::MessageUpdate(gc)
                };

                log::debug!("Message Create/update");
                Broker::<SystemBroker>::issue_async(event);
            }
            "CHANNEL_UPDATE" | "CHANNEL_CREATE" | "CHANNEL_DELETE" => {
                //  log::info!("{}\n{}", event_name, gateway_message);
                let gc: GuildChannel = serde_json::from_value(gateway_message)?;
                let event = if event_name == "CHANNEL_CREATE" {
                    ChannelEvent::ChannelCreate(gc)
                } else if event_name == "CHANNEL_DELETE" {
                    ChannelEvent::ChannelUpdate(gc)
                } else {
                    ChannelEvent::ChannelDelete(gc)
                };
                Broker::<SystemBroker>::issue_async(event);
            }

            &_ => {
                log::warn!("Unknown event {}\n{}", event_name, gateway_message)
            }
        }
        Ok((true, None))
    }
    async fn handle_ws(
        &mut self,
        //  connection: &mut actix_codec::Framed<BoxedSocket, Codec>,
        response: Frame,
    ) -> Result<(bool, Option<Message>)> {
        match response {
            Frame::Text(txt) => {
                let b: GatewayMessage = serde_json::from_str(&String::from_utf8_lossy(&txt))?;
                if let Some(new_sequence) = b.s {
                    self.sequence_number = Some(new_sequence);
                }
                match b.op {
                    gateway::GATEWAY => {
                        if let Some(gateway_event_name) = b.t {
                            return self.handle_ws_gateway_event(&gateway_event_name, b.d).await;
                        } else {
                            log::warn!("Gateway No Event ?? {}", &String::from_utf8_lossy(&txt));
                        }
                    }
                    gateway::HELLO => {
                        let hello: GatewayHello = serde_json::from_value(b.d)?;
                        log::info!("Heartbeat:{}ms", hello.heartbeat_interval);
                        self.duration = Duration::from_millis(hello.heartbeat_interval);
                        self.interval = tokio::time::interval(self.duration);
                        let identify = serde_json::to_value(GatewayIdentify::create(
                            &self.api.token,
                            self.intents,
                        ))?;
                        let msg_json: String = serde_json::to_string(&GatewayMessage {
                            op: gateway::IDENTIFY,
                            d: identify,
                            s: None,
                            t: None,
                        })?;
                        log::info!("Identify");
                        let message = Message::Text(msg_json.into());
                        return Ok((true, Some(message)));
                    }
                    gateway::ACK => {
                        log::debug!("ACKED {}", String::from_utf8_lossy(&txt));
                    }
                    gateway::INVALID_SESSION => {
                        log::info!("INVALID session {}", b.d.as_bool().unwrap_or(false));
                    }
                    _ => {
                        log::error!("Unknown Op Code: {}", b.op)
                    }
                }
            }
            Frame::Binary(_) => {}
            Frame::Continuation(_) => {}
            Frame::Ping(p) => {
                log::info!("Ping");
                let pong = Message::Pong(p);
                return Ok((true, Some(pong)));
                //connection.send(pong).await?;
            }
            Frame::Pong(_) => {}
            Frame::Close(b) => {
                match b {
                    Some(close) => {
                        log::warn!(
                            "Socket Closed xx/{}",
                            // close.code.into(),
                            close.description.unwrap_or_default()
                        )
                    }
                    None => {
                        log::warn!("Socket Closed no-reason")
                    }
                };
                return Ok((false, None));
            }
        }
        Ok((true, None))
    }
    pub async fn start_websocket(&mut self) -> Result<()> {
        let mut connect_ws = self.web_socket.clone();
        connect_ws.set_query(Some("v=9&encoding=json&compress=false"));
        log::info!("Starting Connect {}", connect_ws.as_str());

        let (_resp, mut connection) = self.client.ws(connect_ws.as_str()).connect().await.unwrap();
        // let mut interval = tokio::time::interval(duration);
        Broker::<SystemBroker>::issue_async(Event::INIT);
        loop {
            log::debug!("Starting Select");
            tokio::select! {
                websocket = connection.next() => {
                    log::debug!("WS has a message");
                    let response = websocket.unwrap().unwrap();
                    let (continu,message_send) = self.handle_ws(response).await?;
                    if let Some(to_be_sent) = message_send {
                         let _result =  connection.send(to_be_sent).await?;
                    }
                    if !continu {
                        break
                    }

                }
                _ =  self.interval.tick() => {
                    let heartbeat = serde_json::to_value(self.sequence_number)?;
                    let msg_json : String = serde_json::to_string( &GatewayMessage{ op:gateway::HEARTBEAT, d:heartbeat,s:None,t:None})?;
                    log::debug!("Sending Heart-beart {}", msg_json);
                    let message= Message::Text(msg_json.into());
                    let _result = connection.send(message).await?;
                }
            }
            log::debug!("end-of-loop");
        }

        Ok(())
    }
}
