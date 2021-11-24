use crate::types::*;
use prost::Message;
use crate::validating::{
	decrypt_message, is_user_logged_in,
	aes_encrypt_local, aes_decrypt_local,
};
use std::collections::{HashMap, VecDeque};
use tea_actor_utility::actor_raft::{
	raft_delete_value, raft_get_value, raft_get_values, raft_set_value,
};
use tea_actor_utility::{
	actor_crypto::{
		aes_decrypt, aes_encrypt,
	}
};
use tea_actor_utility::common::calculate_hash;
use tea_actor_utility::actor_env::current_timestamp;
use tea_codec::{deserialize, serialize};
use tea_codec;

use vmh_codec::message::{
	structs_proto::{orbitdb},
	encode_protobuf,
};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;

const MESSAGE_STORAGE_INDEX: u32 = 1;
const MESSAGE_HISTORY_INDEX: u32 = 2;

pub const MAX_HISTORY_MESSAGES_LINES: usize = 20;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelHistoryIndex {
	messages: VecDeque<u64>,
}

impl Default for ChannelHistoryIndex {
	fn default() -> Self {
		ChannelHistoryIndex {
			messages: Default::default(),
		}
	}
}

pub(crate) fn check_user_login(address: &str) -> anyhow::Result<()> {
	if !is_user_logged_in(address)? {
		return Err(anyhow::anyhow!(
			"user is not logged in, please login first"
		));
	}

	Ok(())
}

pub(crate) fn post_message(
	uuid: &str, 
	request: PostMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	// check_user_login(&request.address)?;

	// if is_host_locally(request.tapp_id) {
	// 	return post_message_locally(uuid, request);
	// }

	// todo find host node and relay post message

	// to orbitdb
	let message = {
		let tmp = decrypt_message(&request.encrypted_message, &request.address)?;
		aes_encrypt_local(&tmp)?
	};

	let now: u64 = current_timestamp()? as u64;

	let ttl: u64 = {
		match is_global_channel(&request.channel) {
			true => (2 * 60 * 60) as u64,
			false => (24 * 60 * 60) as u64,
		}
	};

	let dbname = db_name(request.tapp_id, &request.channel);
	let add_message_data = orbitdb::AddMessageRequest {
		tapp_id: request.tapp_id,
		dbname,
		sender: request.address,
		content: message,
		utc: now,
		utc_expired: now+ttl,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"bbs_AddMessage",
				encode_protobuf(add_message_data)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;
	// info!("[bbs] post_message response: {:?}", res);

	Ok(res.data.into_bytes())
}

pub(crate) fn load_message_list(
	uuid: &str,
	request: LoadMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	// check_user_login(&request.address)?;

	// if is_host_locally(request.tapp_id) {
	// 	return load_message_list_locally(uuid, request);
	// }

	// todo find host node and relay load message, return list asynchronously

	// to orbitdb
	let dbname = db_name(request.tapp_id, &request.channel);
	let get_message_data = orbitdb::GetMessageRequest {
		tapp_id: request.tapp_id,
		dbname,
		sender: match request.address.is_empty() {
			true => "".to_string(),
			false => request.address,
		},
		utc: current_timestamp()? as u64,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"bbs_GetMessage",
				encode_protobuf(get_message_data)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;
	
	let mut rs: Vec<MessageItem> = Vec::new();
	let mut arr: Vec<serde_json::Value>;
	let tmp: serde_json::Value = serde_json::from_str(&res.data)?;
	match tmp.as_array() {
		Some(v) => (arr = v.clone()),
		None => (arr = vec![]),
	}
	
	for item in arr.iter() {
		let text = item["content"].as_str().unwrap().to_string();

		let message_item: MessageItem = MessageItem {
			tapp_id: item["tapp_id"].as_u64().unwrap_or(0 as u64),
			id: item["_id"].as_str().unwrap_or("").to_string(),
			sender: item["sender"].as_str().unwrap().to_string(),
			utc: item["utc"].as_u64().unwrap(),
			utc_expired: item["utc_expired"].as_u64().unwrap(),
			content: aes_decrypt_local(&text)?,
		};

		// info!("222222=====> {:?}", message_item);
		rs.push(message_item);
	}

	Ok(serde_json::to_string(&rs)?.into_bytes())
}

pub(crate) fn extend_message(
	_uuid: &str,
	request: ExtendMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	let utc_expired = {
		let now: u64 = current_timestamp()? as u64;
		let ttl: u64 = (1 * 60 * 60) as u64;
		match request.time {
			Some(v) => (v as u64),
			None => ttl,
		}
	};

	let dbname = db_name(request.tapp_id, &request.channel);
	let extend_message_data = orbitdb::ExtendMessageRequest {
		tapp_id: request.tapp_id,
		dbname,
		msg_id: request.msg_id,
		utc_expired,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"bbs_ExtendMessage",
				encode_protobuf(extend_message_data)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;
	// info!("[bbs] extend message response: {:?}", res);

	Ok(res.data.into_bytes())
}

pub(crate) fn delete_message(
	_uuid: &str,
	request: DeleteMessageRequest,
) -> anyhow::Result<Vec<u8>> {

	let dbname = db_name(request.tapp_id, &request.channel);
	let delete_message_data = orbitdb::DeleteMessageRequest {
		tapp_id: request.tapp_id,
		dbname,
		msg_id: request.msg_id,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"bbs_DeleteMessage",
				encode_protobuf(delete_message_data)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;
	// info!("[bbs] delete message response: {:?}", res);

	Ok(res.data.into_bytes())
}

pub(crate) fn post_message_locally(
	uuid: &str,
	request: PostMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	let message = decrypt_message(&request.encrypted_message, &request.address)?;

	let msg_hash = calculate_hash(&message);
	raft_set_value(
		&message_key(&request.channel, msg_hash),
		message.as_bytes(),
		MESSAGE_STORAGE_INDEX,
		uuid,
	)?;

	let mut history = match raft_get_value(&request.channel, MESSAGE_HISTORY_INDEX, uuid) {
		Ok(history_buf) => {
			let mut last_history: ChannelHistoryIndex = deserialize(&history_buf)?;
			while last_history.messages.len() >= MAX_HISTORY_MESSAGES_LINES {
				let hash = last_history.messages.pop_front();
				if let Some(hash) = hash {
					raft_delete_value(
						&message_key(&request.channel, hash),
						MESSAGE_STORAGE_INDEX,
						uuid,
					)?;
				}
			}
			last_history
		}
		Err(e) => {
			warn!(
				"get channel '{}' history object failed: {}",
				request.channel, e
			);
			Default::default()
		}
	};

	history.messages.push_back(msg_hash);
	raft_set_value(
		&request.channel,
		&serialize(history)?,
		MESSAGE_HISTORY_INDEX,
		uuid,
	)?;

	Ok(serde_json::to_string(&PostMessageResponse { success: true })?.into_bytes())
}

pub(crate) fn load_message_list_locally(
	uuid: &str,
	request: LoadMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	// todo implement async logic later

	let values: HashMap<String, Vec<u8>> =
		raft_get_values(&request.channel, MESSAGE_STORAGE_INDEX, uuid)?;
	let message_histories: ChannelHistoryIndex = deserialize(&raft_get_value(
		&request.channel,
		MESSAGE_HISTORY_INDEX,
		uuid,
	)?)?;

	let mut messages: Vec<String> = Vec::new();
	for hash in message_histories.messages.iter() {
		match values.get(&message_key(&request.channel, *hash)) {
			Some(msg) => {
				messages.push(String::from_utf8(msg.clone())?);
			}
			None => {
				error!("failed to get message with hash: {}", hash);
			}
		}
	}

	Ok(serde_json::to_string(&LoadMessageResponse { messages })?.into_bytes())
}

fn is_host_locally(_tapp_id: u64) -> bool {
	// todo get from layer1
	true
}

fn message_key(channel: &str, hash: u64) -> String {
	format!("{}:{}", channel, hash)
}

fn db_name(tapp_id: u64, channel: &str) -> String {
	if is_global_channel(&channel) {
		return channel.to_string();
	} 

	format!("{}_{}", tapp_id, channel)
}

fn is_global_channel(channel: &str) -> bool {
	"test" == channel
}
