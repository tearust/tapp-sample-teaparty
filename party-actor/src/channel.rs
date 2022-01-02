use crate::state;
use crate::types::*;
use crate::validating::{aes_decrypt_local, aes_encrypt_local, is_user_logged_in};
use interface::AuthKey;
use prost::Message;
use std::collections::{HashMap, VecDeque};
use tea_actor_utility::actor_crypto::{aes_decrypt, aes_encrypt};
use tea_actor_utility::actor_env::current_timestamp;
use tea_actor_utility::actor_raft::{
	raft_delete_value, raft_get_value, raft_get_values, raft_set_value,
};
use tea_actor_utility::common::calculate_hash;
use tea_codec;
use tea_codec::{deserialize, serialize};

use vmh_codec::message::{encode_protobuf, structs_proto::orbitdb};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;

use crate::user;

const MESSAGE_STORAGE_INDEX: u32 = 1;
const MESSAGE_HISTORY_INDEX: u32 = 2;

pub const MAX_HISTORY_MESSAGES_LINES: usize = 20;

pub(crate) fn check_user_login(address: &str) -> anyhow::Result<()> {
	if !is_user_logged_in(address)? {
		return Err(anyhow::anyhow!("user is not logged in, please login first"));
	}

	Ok(())
}

pub(crate) fn post_message(uuid: &str, req: &PostMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	// to orbitdb
	let message = {
		let msg = base64::decode(&req.encrypted_message)?;

		let aes_key = user::get_aes_key(&req.tapp_id)?;
		let mut data = msg.to_vec();
		if data.len() < 8 {
			data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
		}

		let msg = aes_encrypt(aes_key, msg)?;
		base64::encode(msg)
	};

	let now: u64 = current_timestamp()? as u64;

	let ttl: u64 = {
		match is_global_channel(&req.channel) {
			true => (2 * 60 * 60) as u64,
			false => (24 * 60 * 60) as u64,
		}
	};

	// state
	// state::post_message(&request.address, ttl.clone(), uuid, auth)?;

	let dbname = db_name(req.tapp_id, &req.channel);
	let add_message_data = orbitdb::AddMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		sender: req.address.clone(),
		content: message,
		utc: now,
		utc_expired: now + ttl,
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
	info!("[bbs] post_message response: {:?}", res);

	Ok(res.data.into_bytes())
}

pub(crate) fn load_message_list(
	_uuid: &str,
	request: LoadMessageRequest,
) -> anyhow::Result<Vec<u8>> {
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
	let arr: Vec<serde_json::Value>;
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
		let _now: u64 = current_timestamp()? as u64;
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
