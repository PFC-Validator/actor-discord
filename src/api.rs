use anyhow::Result;
use awc::http::StatusCode;
//use awc::{ws, Client, ClientBuilder};
use awc::{Client, ClientResponse};
use lazy_static::lazy_static;
//use futures_util::{sink::SinkExt as _, stream::StreamExt as _};
use crate::errors::ActorDiscordError;
use crate::types::events::{
    Guild, GuildChannel, GuildChannelCreate, MessageCreate, MessageObject, RetryMessage,
    SnowflakeID,
};
use actix_http::encoding::Decoder;
use actix_http::Payload;
use regex::Regex;
use serde::Deserialize;
use std::str::FromStr;
use url::Url;

const API_PREFIX: &str = "/api/v9/";

const GUILD_ID: &str = "guilds/";
pub struct DiscordAPI {
    pub client: Client,
    pub base_url: Url,
    pub token: String,
    pub max_retries: usize,
}
impl DiscordAPI {
    pub fn create(token: &str, connect_addr: &str, max_retries: usize) -> Result<DiscordAPI> {
        let base_url: Url = Url::from_str(connect_addr)?.join(API_PREFIX)?;
        let client = Client::builder().finish();
        Ok(DiscordAPI {
            client,
            base_url,
            token: token.into(),
            max_retries,
        })
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, url_suffix: &str) -> anyhow::Result<T> {
        let full_url = self.base_url.join(url_suffix)?;

        let mut retries = self.max_retries;
        while retries > 0 {
            log::debug!("Get URL={}", full_url.as_str());
            let response = self
                .client
                .get(full_url.as_str())
                .insert_header((awc::http::header::CONTENT_TYPE, "application/json"))
                .insert_header((awc::http::header::USER_AGENT, "PFC-Discord"))
                .insert_header((
                    awc::http::header::AUTHORIZATION,
                    format!("Bot {}", self.token),
                ))
                .send()
                .await
                .map_err(|source| {
                    eprintln!("{:#?}", source);
                    ActorDiscordError::ResponseError()
                })?;
            let ok_retryable = self.handle_response::<T>(response).await?;
            if ok_retryable.0 {
                return Ok(ok_retryable.1.unwrap());
            }
            log::debug!("Retrying retries left:{}", retries);
            retries -= 1;
        }
        Err(ActorDiscordError::RetryError.into())
    }
    pub async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        url_suffix: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<T> {
        let full_url = self.base_url.join(url_suffix)?;

        let mut retries = self.max_retries;
        while retries > 0 {
            log::debug!("Post URL={}", full_url.as_str());
            let arg_json = serde_json::to_string(&args)?;
            let response = self
                .client
                .post(full_url.as_str())
                .insert_header((awc::http::header::CONTENT_TYPE, "application/json"))
                .insert_header((awc::http::header::USER_AGENT, "PFC-Discord"))
                .insert_header((
                    awc::http::header::AUTHORIZATION,
                    format!("Bot {}", self.token),
                ))
                .send_body(arg_json)
                .await
                .map_err(|source| {
                    eprintln!("{:#?}", source);
                    ActorDiscordError::ResponseError()
                })?;
            let ok_retryable = self.handle_response::<T>(response).await?;
            if ok_retryable.0 {
                return Ok(ok_retryable.1.unwrap());
            }
            log::debug!("Retrying retries left:{}", retries);
            retries -= 1;
        }
        Err(ActorDiscordError::RetryError.into())
    }
    pub async fn delete<T: for<'de> Deserialize<'de>>(
        &self,
        url_suffix: &str,
    ) -> anyhow::Result<T> {
        let full_url = self.base_url.join(url_suffix)?;

        let mut retries = self.max_retries;
        while retries > 0 {
            log::debug!("Delete URL={}", full_url.as_str());

            let response = self
                .client
                .delete(full_url.as_str())
                .insert_header((awc::http::header::CONTENT_TYPE, "application/json"))
                .insert_header((awc::http::header::USER_AGENT, "PFC-Discord"))
                .insert_header((
                    awc::http::header::AUTHORIZATION,
                    format!("Bot {}", self.token),
                ))
                .send()
                .await
                .map_err(|source| {
                    eprintln!("{:#?}", source);
                    ActorDiscordError::ResponseError()
                })?;
            let ok_retryable = self.handle_response::<T>(response).await?;
            if ok_retryable.0 {
                return Ok(ok_retryable.1.unwrap());
            }
            log::debug!("Retrying retries left:{}", retries);
            retries -= 1;
        }
        Err(ActorDiscordError::RetryError.into())
    }
    pub async fn patch<T: for<'de> Deserialize<'de>>(
        &self,
        url_suffix: &str,
        args: serde_json::Value,
    ) -> anyhow::Result<T> {
        let full_url = self.base_url.join(url_suffix)?;

        let mut retries = self.max_retries;
        while retries > 0 {
            log::debug!("Patch URL={}", full_url.as_str());
            let arg_json = serde_json::to_string(&args)?;
            let response = self
                .client
                .patch(full_url.as_str())
                .insert_header((awc::http::header::CONTENT_TYPE, "application/json"))
                .insert_header((awc::http::header::USER_AGENT, "PFC-Discord"))
                .insert_header((
                    awc::http::header::AUTHORIZATION,
                    format!("Bot {}", self.token),
                ))
                .send_body(arg_json)
                .await
                .map_err(|source| {
                    eprintln!("{:#?}", source);
                    ActorDiscordError::ResponseError()
                })?;
            let ok_retryable = self.handle_response::<T>(response).await?;
            if ok_retryable.0 {
                return Ok(ok_retryable.1.unwrap());
            }
            log::debug!("Retrying retries left:{}", retries);
            retries -= 1;
        }
        Err(ActorDiscordError::RetryError.into())
    }

    /**
     check response code, sleeping if required.
       @returns ok=true/retry=false , or Error
    if OK returns 'T' as 2nd parameter
    */
    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        mut response: ClientResponse<Decoder<Payload>>,
    ) -> Result<(bool, Option<T>)> {
        if response.status() == StatusCode::CREATED || response.status() == StatusCode::OK {
            let result: T = response.json::<T>().limit(1024 * 1024).await?;
            return Ok((true, Some(result)));
        }
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            let retry: RetryMessage = response.json::<RetryMessage>().await?;
            log::debug!(
                "Sleeping for {} seconds :{}",
                retry.retry_after,
                retry.message
            );
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(retry.retry_after)).await;
            return Ok((false, None));
        }
        log::error!(
            "{} {}",
            response.status(),
            std::str::from_utf8(&response.body().limit(6000).await.unwrap())?
        );
        Err(ActorDiscordError::ResponseError().into())
    }
    pub async fn guild(&self, id: SnowflakeID) -> Result<Guild> {
        let url = self.base_url.join(GUILD_ID)?.join(&id.to_string())?;
        let guild: Guild = self.get(url.as_str()).await?;
        Ok(guild)
    }
    pub async fn channels(&self, guild_id: SnowflakeID) -> Result<Vec<GuildChannel>> {
        let prefix = format!("{}{}/channels", GUILD_ID, guild_id.to_string());
        let url = self.base_url.join(&prefix)?;
        let channels: Vec<GuildChannel> = self.get(url.as_str()).await?;
        Ok(channels)
    }
    pub async fn create_channel(
        &self,
        guild_id: SnowflakeID,
        channel_details: GuildChannelCreate,
    ) -> Result<GuildChannel> {
        let prefix = format!("{}{}/channels", GUILD_ID, guild_id.to_string());
        //   let url = self.base_url.join(&prefix)?;
        self.post(&prefix, serde_json::to_value(&channel_details)?)
            .await
    }
    pub async fn delete_channel(&self, channel_id: SnowflakeID) -> Result<GuildChannel> {
        let prefix = format!("channels/{}", channel_id.to_string());
        //   let url = self.base_url.join(&prefix)?;
        self.delete(&prefix).await
    }
    pub async fn patch_channel(
        &self,
        channel_id: SnowflakeID,
        args: serde_json::Value,
    ) -> Result<GuildChannel> {
        let prefix = format!("channels/{}", channel_id.to_string());
        //   let url = self.base_url.join(&prefix)?;
        self.patch(&prefix, args).await
    }
    pub fn sanitize(source: &str) -> String {
        //  let mut lowercase = source.to_ascii_lowercase();
        lazy_static! {
            static ref RE: Regex = Regex::new(r"[^\pN\p{Emoji}A-Za-z0-9\-]").unwrap();
            static ref RE_HASH: Regex = Regex::new(r"#").unwrap();
            static ref RE_DUP: Regex = Regex::new(r"-+").unwrap();
            static ref RE_START: Regex = Regex::new(r"^-").unwrap();
            static ref RE_END: Regex = Regex::new(r"-$").unwrap();
        }

        let sanitized = RE.replace_all(source, "-").to_string();
        let de_hash = RE_HASH.replace_all(&sanitized, "-").to_string();
        let de_dup: String = RE_DUP.replace_all(&de_hash, "-").to_string();
        let trimmed_start: String = RE_START.replace_all(&de_dup, "").to_string();
        let trimmed_end: String = RE_END.replace_all(&trimmed_start, "").to_string();
        trimmed_end.to_lowercase()
    }
    pub async fn create_message(
        &self,
        channel_id: SnowflakeID,
        message: MessageCreate,
    ) -> Result<MessageObject> {
        let prefix = format!("channels/{}/messages", channel_id.to_string());
        //   let url = self.base_url.join(&prefix)?;
        let args = serde_json::to_value(message)?;
        self.post(&prefix, args).await
    }
}
#[cfg(test)]
mod tests {
    use crate::DiscordAPI;

    #[test]
    fn sanitize() {
        let match_tests: Vec<(&str, &str)> = vec![
            ("a", "a"),
            ("b b", "b-b"),
            ("-c", "c"),
            ("--d--", "d"),
            ("@#$a", "a"),
            ("TB 🚀 🌕 b 🔥 L", "tb-🚀-🌕-b-🔥-l"),
        ];
        for t in match_tests {
            let result = DiscordAPI::sanitize(t.0);
            assert_eq!(t.1, result)
        }
    }
}
