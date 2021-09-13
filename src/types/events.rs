use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::*;

#[derive(Debug, Clone, Deserialize, Serialize, actix::Message)]
//#[rtype(result = "Result<usize, ()>")]
#[rtype(result = "()")]

pub enum Event {
    INIT,
    GuildCreate(GuildCreate),
}
#[derive(Debug, Clone, Deserialize, Serialize, actix::Message)]
#[rtype(result = "()")]
pub enum MessageEvent {
    MessageCreate(MessageObject),
    MessageUpdate(MessageObject),
    MessageDelete(MessageObject),
}
#[derive(Debug, Clone, Deserialize, Serialize, actix::Message)]
#[rtype(result = "()")]
pub enum ChannelEvent {
    ChannelCreate(GuildChannel),
    ChannelUpdate(GuildChannel),
    ChannelDelete(GuildChannel),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SnowflakeID {
    pub id: u64,
}
impl Serialize for SnowflakeID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.id.to_string())
    }
}
impl<'de> Deserialize<'de> for SnowflakeID {
    fn deserialize<D>(deserializer: D) -> Result<SnowflakeID, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        match s.parse::<u64>() {
            Err(_e) => {
                eprintln!("u64 Fail {} {:#?}", s, _e);
                Err(serde::de::Error::custom(_e))
            }
            Ok(val) => Ok(SnowflakeID { id: val }),
        }
    }
}
impl From<u64> for SnowflakeID {
    fn from(id: u64) -> Self {
        SnowflakeID { id }
    }
}
impl From<&str> for SnowflakeID {
    fn from(id: &str) -> Self {
        let num: u64 = id.parse().unwrap_or(0);
        SnowflakeID { id: num }
    }
}
impl ToString for SnowflakeID {
    fn to_string(&self) -> String {
        format!("{}", self.id)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildCreate {
    pub id: SnowflakeID,
    pub name: Option<String>,
    pub owner_id: SnowflakeID,
    pub channels: Vec<GuildChannel>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Guild {
    pub id: SnowflakeID,
    pub name: Option<String>,
    pub owner_id: SnowflakeID,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum ChannelType {
    GuildText = 0,
    DM = 1,
    GuildVoice = 2,
    GroupDm = 3,
    GuildCategory = 4,
    GuildNews = 5,
    GuildStore = 6,
    GuildNewsThread = 10,
    GuildPublicThread = 11,
    GuildPrivateThread = 12,
    GuildStageVoice = 13,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildChannel {
    pub id: SnowflakeID,
    #[serde(rename = "type")]
    pub u_type: ChannelType,
    pub name: String,
    pub position: usize,
    pub topic: Option<String>,
    pub parent_id: Option<SnowflakeID>,
    pub last_message_id: Option<SnowflakeID>,
    pub guild_id: Option<SnowflakeID>,
    pub guild_hashes: Option<GuildHashes>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildChannelCreate {
    #[serde(rename = "type")]
    pub u_type: ChannelType,
    pub name: String,
    pub topic: Option<String>,
    pub bitrate: Option<usize>,
    pub user_limit: Option<usize>,
    pub rate_limit_per_user: usize,
    pub position: usize,
    pub parent_id: Option<SnowflakeID>,
    pub nsfw: bool,
}
impl GuildChannelCreate {
    pub fn simple(
        u_type: ChannelType,
        name: &str,
        topic: Option<String>,
        parent_id: Option<SnowflakeID>,
    ) -> Self {
        GuildChannelCreate {
            u_type,
            name: name.into(),
            topic,
            bitrate: None,
            user_limit: None,
            rate_limit_per_user: 0,
            position: 0,
            parent_id,
            nsfw: false,
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hash {
    pub hash: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GuildHashes {
    pub channels: Hash,
    pub metadata: Hash,
    pub roles: Hash,
    pub version: usize,
}

#[derive(Debug, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum MessageObjectType {
    Default = 0,
    RecipientAdd = 1,
    RecipientRemove = 2,
    Call = 3,
    ChannelNameChange = 4,
    ChannelIconChange = 5,
    ChannelPinnedMessage = 6,
    GuildMemberJoin = 7,
    UserPremiumGuildSubscription = 8,
    UserPremiumGuildSubscriptionTier1 = 9,
    UserPremiumGuildSubscriptionTier2 = 10,
    UserPremiumGuildSubscriptionTier3 = 11,
    ChannelFollowAdd = 12,
    GuildDiscoveryDisqualified = 14,
    GuildDiscoveryRequalified = 15,
    GuildDiscoveryGracePeriodInitialWarning = 16,
    GuildDiscoveryGracePeriodFinalWarning = 17,
    ThreadCreated = 18,
    Reply = 19,
    ChatInputCommand = 20,
    ThreadStarterMessage = 21,
    GuildInviteReminder = 22,
    ContextMenuCommand = 23,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageObject {
    pub id: SnowflakeID,
    #[serde(rename = "type")]
    pub u_type: MessageObjectType,
    pub content: String,
    pub tts: bool,
    pub referenced_message: Box<Option<MessageObject>>,
    pub nonce: Option<SnowflakeID>,
    pub channel_id: Option<SnowflakeID>,
    pub author: UserObject,
    pub mention_everyone: bool,
    pub mentions: Vec<UserObject>,
    pub mention_roles: Vec<SnowflakeID>,
    pub message_reference: Option<MessageReference>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageReference {
    pub message_id: Option<SnowflakeID>,
    pub channel_id: Option<SnowflakeID>,
    pub guild_id: Option<SnowflakeID>,
    pub fail_if_not_exists: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserObject {
    pub id: SnowflakeID,
    pub username: String,
    pub discriminator: String,
    pub email: Option<String>,
    pub bot: Option<bool>,
    pub system: Option<bool>,
    pub flags: Option<u64>,
    pub public_flags: Option<u64>,
}
