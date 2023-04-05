use anyhow::Result;
use std::fmt::{Debug, Display};
use std::path::PathBuf;

pub trait EmoteIdExt
where
    Self: Sized,
{
    fn parse_id(input: &str) -> Result<Self>;

    fn to_url(&self) -> String;
    fn to_file_name(&self) -> PathBuf;
}

#[derive(Clone, Copy)]
pub struct SevenTvId([u8; 12]);

impl Display for SevenTvId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
impl Debug for SevenTvId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SevenTvId")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl EmoteIdExt for SevenTvId {
    fn parse_id(input: &str) -> Result<Self> {
        let mut id = [0u8; 12];
        hex::decode_to_slice(input, &mut id)?;
        Ok(SevenTvId(id))
    }
    fn to_url(&self) -> String {
        format!("https://cdn.7tv.app/emote/{self}/4x.webp")
    }
    fn to_file_name(&self) -> PathBuf {
        PathBuf::from(format!("{self}.webp"))
    }
}

#[derive(Clone, Copy)]
pub struct BttvId([u8; 12]);

impl Display for BttvId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
impl Debug for BttvId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BttvId").field(&hex::encode(self.0)).finish()
    }
}

impl EmoteIdExt for BttvId {
    fn parse_id(input: &str) -> Result<Self> {
        let mut id = [0u8; 12];
        hex::decode_to_slice(input, &mut id)?;
        Ok(BttvId(id))
    }
    fn to_url(&self) -> String {
        format!("https://cdn.betterttv.net/emote/{self}/3x.webp")
    }
    fn to_file_name(&self) -> PathBuf {
        let id = hex::encode(&self.0);
        PathBuf::from(format!("{self}.webp"))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EmoteId {
    SevenTv(SevenTvId),
    Bttv(BttvId),
}

impl From<&SevenTvId> for EmoteId {
    fn from(id: &SevenTvId) -> Self {
        Self::SevenTv(*id)
    }
}

impl From<&BttvId> for EmoteId {
    fn from(id: &BttvId) -> Self {
        Self::Bttv(*id)
    }
}

impl EmoteIdExt for EmoteId {
    fn parse_id(input: &str) -> Result<Self> {
        unimplemented!()
    }
    fn to_file_name(&self) -> PathBuf {
        match self {
            EmoteId::SevenTv(id) => id.to_file_name(),
            EmoteId::Bttv(id) => id.to_file_name(),
        }
    }
    fn to_url(&self) -> String {
        match self {
            EmoteId::SevenTv(id) => id.to_url(),
            EmoteId::Bttv(id) => id.to_url(),
        }
    }
}

impl Display for EmoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmoteId::SevenTv(id) => std::fmt::Display::fmt(&id, f),
            EmoteId::Bttv(id) => std::fmt::Display::fmt(&id, f),
        }
    }
}
