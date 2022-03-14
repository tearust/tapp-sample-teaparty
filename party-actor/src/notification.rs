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
use crate::request::{send_query, send_txn};
use crate::types::*;
use crate::user;
use crate::utility::parse_to_acct;

pub fn add_message(req: &NotificationAddMessageRequest) -> anyhow::Result<Vec<u8>> {
	let uuid = &req.uuid;
	user::check_auth(&req.tapp_id, &req.from, &req.auth_b64)?;

	// send txn
	let block: u32 = help::current_block_number()?;
	let ttl = get_add_message_ttl(&req)?;
	let txn = TeapartyTxn::AddNotificationMessage {
		token_id: req.tapp_id,
		from: parse_to_acct(&req.from)?,
		to: parse_to_acct(&req.to)?,
		current: block,
		ttl,
		auth_b64: req.auth_b64.to_string(),
	};
	let txn_bytes = bincode::serialize(&txn)?;
	send_txn(
		"notification_add_message",
		&uuid,
		bincode::serialize(req)?,
		txn_bytes,
		&tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn add_message_to_db(req: &NotificationAddMessageRequest) -> anyhow::Result<String> {
	// to orbitdb
	let message = {
		let msg = base64::decode(&req.content_b64)?;
		let aes_key = help::get_aes_key(&req.tapp_id)?;
		let mut data = msg.to_vec();
		if data.len() < 8 {
			data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
		}
		let msg = aes_encrypt(aes_key, data)?;
		base64::encode(msg)
	};

	let block: u32 = help::current_block_number()?;
	let ttl: u32 = get_add_message_ttl(&req)?;

	let add_message_data = orbitdb::NotificationAddMessageRequest {
		tapp_id: req.tapp_id,
		from_tapp_id: req.from_tapp_id,
		sender: req.from.clone(),
		to: req.to.clone(),
		content: message,
		utc: block as u64,
		utc_expired: (block + ttl) as u64,
		from_tapp_url: req.from_tapp_url.clone(),
	};
	info!("notification add_message_data => {:?}", &add_message_data);

	let res = orbitdb::OrbitBbsResponse::decode(
		untyped::default()
			.call(
				tea_codec::ORBITDB_CAPABILITY_ID,
				"notification_AddMessage",
				encode_protobuf(add_message_data)?,
			)
			.map_err(|e| anyhow::anyhow!("{}", e))?
			.as_slice(),
	)?;

	info!("[notification] add_message response: {:?}", res);

	Ok(res.data)
}

fn get_add_message_ttl(_req: &NotificationAddMessageRequest) -> anyhow::Result<u32> {
	let ttl: u32 = 1440 * 2;
	Ok(ttl)
}

pub fn get_message_list(req: &NotificationGetMessageRequest) -> anyhow::Result<Vec<u8>> {
	let block = help::current_block_number()?;
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	// to orbitdb
	let get_message_req = orbitdb::NotificationGetMessageRequest {
		utc: (block - 1) as u64,
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

		let aes_key = help::get_aes_key(&req.tapp_id)?;
		let content = aes_decrypt(aes_key, base64::decode(text)?)?;

		let message_item: NotificationMessageItem = NotificationMessageItem {
			tapp_id: item["tapp_id"].as_u64().unwrap_or(0 as u64),
			from_tapp_id: item["from_tapp_id"].as_u64().unwrap_or(0 as u64),
			id: item["_id"].as_str().unwrap_or("").to_string(),
			sender: item["sender"].as_str().unwrap().to_string(),
			to: item["to"].as_str().unwrap().to_string(),
			utc: item["utc"].as_u64().unwrap(),
			utc_expired: item["utc_expired"].as_u64().unwrap(),
			content: String::from_utf8(content)?,
			from_tapp_url: item["from_tapp_url"].as_str().unwrap().to_string(),
		};

		rs.push(message_item);
	}

	Ok(serde_json::to_string(&rs)?.into_bytes())
}
