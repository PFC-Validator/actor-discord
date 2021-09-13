use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActorDiscordError {
    #[error("ResponseError HTTP(s) Error")]
    ResponseError(),
    #[error("Too many retries")]
    RetryError,
}
