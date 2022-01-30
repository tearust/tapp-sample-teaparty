use crate::types::*;
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
use crate::state;
use crate::user;

pub fn add_message(req: &NotificationAddMessageRequest) -> anyhow::Result<Vec<u8>> {
	let uuid = &req.uuid;
	user::check_auth(&req.tapp_id, &req.from, &req.auth_b64)?;

	// to orbitdb
	let message = {
		let msg = base64::decode(&req.content_b64)?;
		let aes_key = user::get_aes_key(&req.tapp_id)?;
		let mut data = msg.to_vec();
		if data.len() < 8 {
			data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
		}
		let msg = aes_encrypt(aes_key, data)?;
		base64::encode(msg)
	};

	let block: u32 = help::current_block_number()?;
	let ttl: u32 = 1440 * 2;

	let can_post_uuid = help::uuid_cb_key(&uuid, &"notification_add_message");

	let add_message_data = orbitdb::NotificationAddMessageRequest {
		tapp_id: req.tapp_id,
		sender: req.from.clone(),
		to: req.to.clone(),
		content: message,
		height_sent: block,
		height_expired: block + ttl,
	};
	info!("notification add_message_data => {:?}", &add_message_data);
	help::set_mem_cache(&can_post_uuid, encode_protobuf(add_message_data)?)?;

	// send txn
	let txn = TeapartyTxn::AddNotificationMessage {
		token_id: req.tapp_id,
		from: state::parse_to_acct(&req.from)?,
		to: state::parse_to_acct(&req.to)?,
		ttl,
		auth_b64: req.auth_b64.to_string(),
	};
	info!("notification add_message txn => {:?}", txn);
	let txn_bytes = bincode::serialize(&txn)?;
	state::execute_tx_with_txn_bytes(
		txn_bytes,
		can_post_uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;
	info!("send txn finish...");

	Ok(b"ok".to_vec())
}

pub fn libp2p_msg_cb(body: &tokenstate::StateReceiverResponse) -> anyhow::Result<bool> {
	let uuid = &body.uuid;

	if uuid.starts_with_ignore_ascii_case("notification_add_message") {
		// add_message cb
		if let Ok(add_message_buf) = help::get_mem_cache(&uuid) {
			if body.msg.is_some() {
				let res = orbitdb::OrbitBbsResponse::decode(
					untyped::default()
						.call(
							tea_codec::ORBITDB_CAPABILITY_ID,
							"notification_AddMessage",
							add_message_buf,
						)
						.map_err(|e| anyhow::anyhow!("{}", e))?
						.as_slice(),
				)?;
				info!("[notification] add_message response: {:?}", res);

				help::set_mem_cache(
					&help::cb_key_to_uuid(uuid, "notification_add_message"),
					encode_protobuf(res)?,
				)?;

				return Ok(true);
			}
		}
	}

	Ok(false)
}

pub fn get_message_list(req: &NotificationGetMessageRequest) -> anyhow::Result<Vec<u8>> {
	let block = help::current_block_number()?;

	// to orbitdb
	let get_message_req = orbitdb::NotificationGetMessageRequest {
		block_height: block - 1,
		sender: match &req.from {
			Some(v) => v.to_string(),
			None => "".to_string(),
		},
		to: match &req.to {
			Some(v) => v.to_string(),
			None => "".to_string(),
		},
	};
	info!("notification get_message_req => {:?}", &get_message_req);

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"notification_GetMessage",
				encode_protobuf(get_message_req)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;

	let mut rs: Vec<NotificationMessageItem> = Vec::new();
	let arr: Vec<serde_json::Value>;
	let tmp: serde_json::Value = serde_json::from_str(&res.data)?;
	match tmp.as_array() {
		Some(v) => (arr = v.clone()),
		None => (arr = vec![]),
	}

	for item in arr.iter() {
		let text = item["content"].as_str().unwrap().to_string();

		let aes_key = user::get_aes_key(&req.tapp_id)?;
		let content = aes_decrypt(aes_key, base64::decode(text)?)?;

		let message_item: NotificationMessageItem = NotificationMessageItem {
			tapp_id: item["tapp_id"].as_u64().unwrap_or(0 as u64),
			id: item["_id"].as_str().unwrap_or("").to_string(),
			sender: item["sender"].as_str().unwrap().to_string(),
			to: item["to"].as_str().unwrap().to_string(),
			utc: item["utc"].as_u64().unwrap(),
			utc_expired: item["utc_expired"].as_u64().unwrap(),
			content: String::from_utf8(content)?,
		};

		rs.push(message_item);
	}

	Ok(serde_json::to_string(&rs)?.into_bytes())
}
