use crate::state;
use crate::types::*;
use crate::validating::{aes_decrypt_local, aes_encrypt_local, is_user_logged_in};
use interface::AuthKey;
use prost::Message;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use str_utils::*;
use tea_actor_utility::actor_crypto::{aes_decrypt, aes_encrypt};
use tea_actor_utility::actor_env::current_timestamp;
use tea_actor_utility::actor_raft::{
	raft_delete_value, raft_get_value, raft_get_values, raft_set_value,
};
use tea_actor_utility::common::calculate_hash;
use tea_codec;
use tea_codec::{deserialize, serialize};

use party_shared::TeapartyTxn;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{orbitdb, tokenstate},
};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;

use crate::help;
use crate::user;

pub fn post_message(uuid: &str, req: &PostMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	// to orbitdb
	let message = {
		let msg = base64::decode(&req.encrypted_message)?;

		let aes_key = user::get_aes_key(&req.tapp_id)?;
		let mut data = msg.to_vec();
		if data.len() < 8 {
			data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
		}
		let msg = aes_encrypt(aes_key, data)?;
		base64::encode(msg)
	};

	let block = help::current_block_number()? as u64;
	// let block: u64 = current_timestamp()? as u64;

	let ttl: u64 = {
		match is_global_channel(&req.channel) {
			true => (2 * 1440) as u64,
			false => (10 * 1440) as u64,
			// true => (2 * 60 * 60) as u64,
			// false => (24 * 60 * 60) as u64,
		}
	};

	let can_post_uuid = help::uuid_cb_key(&uuid, &"state_post_message");
	info!("state begin to post_message => {}", can_post_uuid);
	let txn = TeapartyTxn::PostMessage {
		token_id: req.tapp_id,
		from: state::parse_to_acct(&req.address)?,
		ttl,
		auth_b64: req.auth_b64.to_string(),
	};
	let txn_bytes = bincode::serialize(&txn)?;
	state::execute_tx_with_txn_bytes(
		txn_bytes,
		can_post_uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;
	info!("state post message success");

	let dbname = db_name(req.tapp_id, &req.channel);
	let add_message_data = orbitdb::AddMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		sender: req.address.clone(),
		content: message,
		utc: block,
		utc_expired: block + ttl,
	};
	info!("aaa => {:?}", &add_message_data);
	help::set_mem_cache(&can_post_uuid, encode_protobuf(add_message_data)?)?;

	Ok(b"ok".to_vec())
}

// fn post_message_cb() -> anyhow::Result<()> {

// Ok(res.data.into_bytes())
// }

pub fn libp2p_msg_cb(body: &tokenstate::StateReceiverResponse) -> anyhow::Result<bool> {
	let uuid = &body.uuid;

	if uuid.starts_with_ignore_ascii_case("state_post_message") {
		// post_message cb
		if let Ok(add_message_buf) = help::get_mem_cache(&uuid) {
			if body.msg.is_some() {
				let res = orbitdb::OrbitBbsResponse::decode(
					untyped::default()
						.call(
							tea_codec::ORBITDB_CAPABILITY_ID,
							"bbs_AddMessage",
							add_message_buf,
						)
						.map_err(|e| anyhow::anyhow!("{}", e))?
						.as_slice(),
				)?;
				info!("[bbs] post_message response: {:?}", res);

				help::set_mem_cache(
					&help::cb_key_to_uuid(uuid, "state_post_message"),
					encode_protobuf(res)?,
				)?;

				return Ok(true);
			}
		}
	}

	Ok(false)
}

pub(crate) fn load_message_list(
	_uuid: &str,
	request: LoadMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	let block = help::current_block_number()? as u64;

	// to orbitdb
	let dbname = db_name(request.tapp_id, &request.channel);
	let get_message_data = orbitdb::GetMessageRequest {
		tapp_id: request.tapp_id,
		dbname,
		sender: match request.address.is_empty() {
			true => "".to_string(),
			false => request.address,
		},
		utc: block - 2,
	};
	info!("bbb => {:?}", &get_message_data);
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

		let aes_key = user::get_aes_key(&request.tapp_id)?;
		let content =
			aes_decrypt(aes_key, base64::decode(text)?).unwrap_or(b"Failed to decrypt.".to_vec());

		let message_item: MessageItem = MessageItem {
			tapp_id: item["tapp_id"].as_u64().unwrap_or(0 as u64),
			id: item["_id"].as_str().unwrap_or("").to_string(),
			sender: item["sender"].as_str().unwrap().to_string(),
			utc: item["utc"].as_u64().unwrap(),
			utc_expired: item["utc_expired"].as_u64().unwrap(),
			content: String::from_utf8(content)?,
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
