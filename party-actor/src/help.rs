use interface::Tsid;
use prost::Message;

use base64;
use interface::sql::Payload;
use serde_json::json;
use tea_actor_utility::{
	actor_enclave::{generate_uuid, get_my_tea_id},
	actor_env::{get_system_time, time_since},
	actor_kvp,
	actor_layer1::general_remote_request,
	actor_libp2p,
};
use tea_codec;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{layer1, libp2p, orbitdb, tappstore, tokenstate},
};
use wascc_actor::untyped;

use crate::types::BINDING_NAME;
use crate::user;

pub fn set_mem_cache(key: &str, val: Vec<u8>) -> anyhow::Result<()> {
	actor_kvp::set(BINDING_NAME, &key, &val, 1800).map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}

pub fn get_mem_cache(key: &str) -> anyhow::Result<Vec<u8>> {
	let rs: Vec<u8> = actor_kvp::get(BINDING_NAME, &key)?
		.ok_or(anyhow::anyhow!("failed to get value with {}", key))?;

	Ok(rs)
}

pub fn del_mem_cache(key: &str) -> anyhow::Result<()> {
	actor_kvp::del(BINDING_NAME, &key)?;
	Ok(())
}

const CURRENT_BLOCK_NUMBER_KEY: &str = "tea_tapp_party_actor";
pub(crate) fn persist_current_block(event: &layer1::NewBlockEvent) -> anyhow::Result<()> {
	actor_kvp::set_forever(
		BINDING_NAME,
		CURRENT_BLOCK_NUMBER_KEY,
		&event.block_number,
	)?;
	Ok(())
}

pub(crate) fn current_block_number() -> anyhow::Result<u32> {
	let block_number: u32 =
		actor_kvp::get(BINDING_NAME, CURRENT_BLOCK_NUMBER_KEY)?.unwrap_or_default();
	Ok(block_number)
}

pub fn save_session_key(session_key: String, tapp_id: &u64, address: &str) -> anyhow::Result<()> {
	let key = format!("session_key_{}_{}", tapp_id, address);

	actor_kvp::set(BINDING_NAME, &key, &session_key, 60 * 60 * 1)
		.map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}
pub fn get_session_key(tapp_id: &u64, address: &str) -> anyhow::Result<String> {
	let key = format!("session_key_{}_{}", tapp_id, address);

	let session_key: String = actor_kvp::get(BINDING_NAME, &key)?
		.ok_or(anyhow::anyhow!("failed to get session key"))?;

	Ok(session_key)
}

pub fn save_aes_key(aes_key: Vec<u8>, tapp_id: &u64) -> anyhow::Result<()> {
	let key = format!("aes_key_{}", tapp_id);

	actor_kvp::set_forever(BINDING_NAME, &key, &aes_key)
		.map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}
pub fn get_aes_key(tapp_id: &u64) -> anyhow::Result<Vec<u8>> {
	let key = format!("aes_key_{}", tapp_id);

	let aes_key: Vec<u8> = actor_kvp::get(BINDING_NAME, &key)?
		.ok_or(anyhow::anyhow!("failed to get aes key"))?;

	Ok(aes_key)
}
