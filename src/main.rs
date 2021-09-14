use actix::Actor;
//use actix_rt::Runtime;
use actor_discord::discord::ExampleDiscordActor;
use actor_discord::types::events::{ChannelType, GuildChannelCreate};
use actor_discord::DiscordAPI;
use actor_discord::DiscordBot;
use actor_discord::GatewayIntents;
use anyhow::Result;
use dotenv::dotenv;
use std::env;

#[actix_rt::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    log::info!("Starting");

    let token = env::var("DISCORD_TOKEN")?;
    let url = env::var("DISCORD_URL")?;
    let retries: usize = env::var("DISCORD_RETRIES").unwrap_or("4".into()).parse()?;
    let intents: GatewayIntents = GatewayIntents::GUILDS
         // | GatewayIntents::GUILD_MESSAGE_TYPING
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS;
    log::info!("** Intents = {}", intents.bits);

    log::info!("attempting to create websocket");
    let discord_api = DiscordAPI::create(&token, &url, retries)?;
    // let mut connect = DiscordBot::create(&discord_api, intents).await?;
    log::info!("attempting to create actor");

    //let actor = ExampleDiscordActor::create(&token, &url)?;

    log::info!("attempting to start actor");
    //let _actor_addr = actor.start();
    log::info!("creating threads");

    let channels = discord_api.channels("839604684573638696".into()).await?;
    let foo = channels
        .iter()
        .filter(|c| c.parent_id.is_none() && c.u_type == ChannelType::GuildText)
        .collect::<Vec<_>>();
    let bb = futures::future::join_all(foo.iter().map(|c| discord_api.delete_channel(c.id))).await;
    bb.iter().for_each(|cr| match cr {
        Ok(gc) => {
            log::info!("{}", gc.id.to_string())
        }
        Err(e) => {
            log::error!("{}", e)
        }
    });

    log::info!("{}", channels.len());
    /*
    let new_channel = discord_api
        .create_channel(
            "839604684573638696".into(),
            GuildChannelCreate::simple(
                ChannelType::GuildText,
                "hello",
                Some("hello there".into()),
                None,
            ),
        )
        .await?;

     */
    //    log::info!("{:#?}", new_channel);
    //let mut _f = connect.start_websocket(); //.await?;
    //_f.await?;
    log::info!("done");
    Ok(())
}
