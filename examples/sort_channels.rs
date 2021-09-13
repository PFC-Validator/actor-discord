use actor_discord::types::events::ChannelType;
use actor_discord::DiscordAPI;
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

    let discord_api = DiscordAPI::create(&token, &url)?;

    let channels = discord_api.channels("839604684573638696".into()).await?;
    let mut foo = channels
        .iter()
        .filter(|c| {
            if let Some(topic) = &c.topic {
                c.parent_id.is_none()
                    && c.u_type == ChannelType::GuildText
                    && topic.starts_with("terravaloper1")
            } else {
                false
            }
        })
        .collect::<Vec<_>>();

    foo.sort_by(|a, b| b.name.cmp(&a.name));

    log::info!("#Channels Total: {}", channels.len());
    let mut i = channels.len();
    for channel in foo {
        log::info!("{} {}", i, channel.name);

        if i != channel.position {
            discord_api
                .patch_channel(
                    channel.id,
                    serde_json::from_str(&format!("{{\"position\":{}}}", i))?,
                )
                .await?;
        }
        i = i - 1;
    }

    log::info!("done");
    Ok(())
}
