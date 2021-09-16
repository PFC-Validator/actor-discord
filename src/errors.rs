use thiserror::Error;

#[derive(Error, Debug)]
pub enum ActorDiscordError {
    #[error("ResponseError HTTP(s) Error")]
    ResponseError(),
    #[error("HTTP(s) Error {url:?} {err:?}")]
    ResponseErrorMsg { url: String, err: String },
    #[error("Too many retries")]
    RetryError,
}
