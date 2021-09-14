use crate::types::events::{ChannelEvent, Event, MessageEvent};
use crate::BrokerType;
use crate::DiscordAPI;
use actix::{Actor, Context, ContextFutureSpawner, Handler, WrapFuture};
use actix_broker::BrokerSubscribe;
use anyhow::Result;

pub struct ExampleDiscordActor {
    pub token: String,
    pub connect_addr: String,
    pub max_retries: usize,
}

impl ExampleDiscordActor {
    pub fn create(token: &str, connect_addr: &str, retries: usize) -> Result<Self> {
        Ok(ExampleDiscordActor {
            token: token.into(),
            connect_addr: connect_addr.into(),
            max_retries: retries,
        })
    }
}
impl Actor for ExampleDiscordActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_sync::<BrokerType, Event>(ctx);
        self.subscribe_sync::<BrokerType, MessageEvent>(ctx);
        self.subscribe_sync::<BrokerType, ChannelEvent>(ctx);
        log::info!("Discord Example Actor Started")
    }
}
impl Handler<Event> for ExampleDiscordActor {
    //type Result = ResponseActFuture<Self, Result<usize, ()>>;
    type Result = ();

    fn handle(&mut self, msg: Event, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Event::INIT => {
                log::info!("IN INIT");
                match DiscordAPI::create(&self.token, &self.connect_addr, self.max_retries) {
                    Ok(api) => {
                        async move {
                            match api.guild("839604684573638696".into()).await {
                                Ok(guild) => {
                                    log::info!("GUID = {}", guild.name.unwrap_or_default())
                                }
                                Err(e) => log::error!("FAIL {}", e),
                            }
                        }
                        .into_actor(self)
                        .spawn(ctx);
                    }
                    Err(e) => log::error!("ERR {}", e),
                };
            }
            Event::GuildCreate(_) => {}
        }
    }
}
impl Handler<MessageEvent> for ExampleDiscordActor {
    type Result = ();
    fn handle(&mut self, msg: MessageEvent, _ctx: &mut Self::Context) {
        match msg {
            MessageEvent::MessageCreate(_) => {}
            MessageEvent::MessageUpdate(_) => {}
            MessageEvent::MessageDelete(_) => {}
        };
        log::info!("MEvent {:?}", msg);
    }
}
impl Handler<ChannelEvent> for ExampleDiscordActor {
    type Result = ();
    fn handle(&mut self, msg: ChannelEvent, _ctx: &mut Self::Context) {
        log::info!("CEvent {:?}", msg);
    }
}
