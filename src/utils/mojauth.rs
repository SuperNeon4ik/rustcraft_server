use crate::LOGGER;
use serde_derive::Deserialize;
use sha1::{Sha1, Digest};
use crate::{SESSION_HOST, crypto::auth_hash::{calc_hash, calc_hash_from_str}};

use super::errors::ObjectResponseError;

#[derive(Deserialize)]
pub struct SessionServerHasJoinedResponse {
    pub id: String,
    pub name: String,
    pub properties: Vec<SessionServerProperty>,
}

#[derive(Deserialize)]
pub struct SessionServerProperty {
    pub name: String,
    pub value: String,
    pub signature: String,
}

pub fn authenticate_player(username: String, shared_secret: &[u8], encoded_public_key: &[u8]) -> Result<SessionServerHasJoinedResponse, ObjectResponseError> {
    let sha = Sha1::new()
        .chain_update("".as_bytes())
        .chain_update(shared_secret)
        .chain_update(encoded_public_key);

    let hash = calc_hash(sha);
    log!(debug, "username = {}; hash = {}", username, hash);
    let endpoint = format!("/session/minecraft/hasJoined?username={}&serverId={}", username, hash);
    let body = reqwest::blocking::get(SESSION_HOST.to_owned() + &endpoint)?.text()?;

    match serde_json::from_str(&body) {
        Ok(result) => Ok(result),
        Err(e) => Err(ObjectResponseError::SerdeParseError(format!("{} ({})", body, e.to_string())))
    }
}