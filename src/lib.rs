use actix_broker::SystemBroker;

mod api;
mod connection;
pub mod discord;
mod errors;
mod intents;
pub mod types;
pub use api::DiscordAPI;
pub use connection::DiscordBot;
pub use intents::GatewayIntents;
/// VERSION number of package
pub const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
/// NAME of package
pub const NAME: Option<&'static str> = option_env!("CARGO_PKG_NAME");
pub type BrokerType = SystemBroker;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
