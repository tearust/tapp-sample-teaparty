use prost::Message;
use tea_actor_utility::actor_crypto::{aes_decrypt, aes_encrypt};
use tea_actor_utility::{wascc_call,};
use tea_codec;

use party_shared::TeapartyTxn;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{orbitdb,},
};


use crate::help;
use crate::request::{send_txn};
use crate::types::*;
use crate::user;
use crate::utility::parse_to_acct;

pub fn post_message(req: &PostMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	let uuid = &req.uuid;
	let ttl = get_post_message_ttl(&req)?;
	let txn = TeapartyTxn::PostMessage {
		token_id: req.tapp_id,
		from: parse_to_acct(&req.address)?,
		ttl,
		auth_b64: req.auth_b64.to_string(),
	};

	let txn_bytes = bincode::serialize(&txn)?;

	send_txn(
		"post_message",
		&uuid,
		bincode::serialize(req)?,
		txn_bytes,
		&tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn post_message_to_db(req: &PostMessageRequest) -> anyhow::Result<String> {
	let message = {
		let msg = base64::decode(&req.encrypted_message)?;

		let aes_key = help::get_aes_key(&req.tapp_id)?;
		let mut data = msg.to_vec();
		if data.len() < 8 {
			data.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
		}
		let msg = aes_encrypt(aes_key, data)?;
		base64::encode(msg)
	};

	let block = help::current_block_number()? as u64;
	let ttl = get_post_message_ttl(&req)?;

	let dbname = db_name(req.tapp_id, &req.channel);
	let add_message_data = orbitdb::AddMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		sender: req.address.clone(),
		content: message,
		utc: block,
		utc_expired: block + ttl,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		wascc_call(
			tea_codec::ORBITDB_CAPABILITY_ID,
			"bbs_AddMessage",
			&encode_protobuf(add_message_data)?,
		)?.as_slice(),
	)?;
	info!("[bbs] post_message response: {:?}", res);

	Ok(res.data)
}

fn get_post_message_ttl(req: &PostMessageRequest) -> anyhow::Result<u64> {
	let ttl: u64 = {
		match is_global_channel(&req.channel) {
			true => 14400 as u64,
			false => {
				if let Some(n) = &req.ttl {
					*n
				} else {
					14400 as u64
				}
			}
		}
	};

	Ok(ttl)
}

pub fn load_message_list(req: &LoadMessageRequest) -> anyhow::Result<Vec<u8>> {
	let block = help::current_block_number()? as u64;

	// to orbitdb
	let dbname = db_name(req.tapp_id, &req.channel);
	let get_message_data = orbitdb::GetMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		sender: match req.address.is_empty() {
			true => "".to_string(),
			false => req.address.to_string(),
		},
		utc: block - 2,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		wascc_call(
			tea_codec::ORBITDB_CAPABILITY_ID,
			"bbs_GetMessage",
			&encode_protobuf(get_message_data)?,
		)?.as_slice(),
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

		let aes_key = help::get_aes_key(&req.tapp_id)?;
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

		rs.push(message_item);
	}

	Ok(serde_json::to_string(&rs)?.into_bytes())
}

pub fn extend_message(req: &ExtendMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	let uuid = &req.uuid;
	let txn = TeapartyTxn::ExtendMessage {
		token_id: req.tapp_id,
		from: parse_to_acct(&req.address)?,
		ttl: req.ttl,
		auth_b64: req.auth_b64.to_string(),
	};

	let txn_bytes = bincode::serialize(&txn)?;
	send_txn(
		"extend_message",
		&uuid,
		bincode::serialize(req)?,
		txn_bytes,
		&tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn extend_message_to_db(req: &ExtendMessageRequest) -> anyhow::Result<()> {
	let dbname = db_name(req.tapp_id, &req.channel);
	let extend_message_data = orbitdb::ExtendMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		msg_id: req.msg_id.to_string(),
		utc_expired: req.ttl,
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		wascc_call(
			tea_codec::ORBITDB_CAPABILITY_ID,
			"bbs_ExtendMessage",
			&encode_protobuf(extend_message_data)?,
		)?.as_slice(),
	)?;
	info!("[bbs] extend message response: {:?}", res);

	// Ok(res.data.into_bytes())
	Ok(())
}

pub fn delete_message(req: &DeleteMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	let uuid = &req.uuid;
	let txn = TeapartyTxn::DeleteMessage {
		token_id: req.tapp_id,
		from: parse_to_acct(&req.address)?,
		auth_b64: req.auth_b64.to_string(),
		is_tapp_owner: req.is_tapp_owner,
	};

	let txn_bytes = bincode::serialize(&txn)?;
	send_txn(
		"delete_message",
		&uuid,
		bincode::serialize(req)?,
		txn_bytes,
		&tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn delete_message_to_db(req: &DeleteMessageRequest) -> anyhow::Result<()> {
	let dbname = db_name(req.tapp_id, &req.channel);
	let delete_message_data = orbitdb::DeleteMessageRequest {
		tapp_id: req.tapp_id,
		dbname,
		msg_id: req.msg_id.to_string(),
	};

	let res = orbitdb::OrbitBbsResponse::decode(
		wascc_call(
			tea_codec::ORBITDB_CAPABILITY_ID,
			"bbs_DeleteMessage",
			&encode_protobuf(delete_message_data)?,
		)?.as_slice(),
	)?;
	info!("[bbs] delete message response: {:?}", res);

	Ok(())
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
