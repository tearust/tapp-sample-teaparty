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
	structs_proto::{orbitdb, replica, tappstore, tokenstate},
};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;

use crate::channel;
use crate::help;
use crate::notification;
use crate::state;
use crate::user;

pub fn sm_txn_request(
	action_name: &str,
	uuid: &str,
	req_bytes: Vec<u8>,
	txn_bytes: Vec<u8>,
	txn_target: &str,
) -> anyhow::Result<()> {
	let ori_uuid = str::replace(&uuid, "txn_", "");
	let action_key = help::uuid_cb_key(&ori_uuid, "action_name");
	let req_key = help::uuid_cb_key(&ori_uuid, "action_req");
	help::set_mem_cache(&action_key, bincode::serialize(&action_name)?)?;
	help::set_mem_cache(&req_key, req_bytes.clone())?;

	info!("start to send txn request for {:?}", &uuid);
	state::execute_tx_with_txn_bytes(txn_bytes, uuid.to_string(), txn_target.to_string())?;
	info!("finish to send txn request...");

	Ok(())
}

pub fn sm_txn_cb(
	req: replica::FindExecutedTxnResponse,
	uuid: &str,
) -> anyhow::Result<serde_json::Value> {
	info!("aaa => {:?}\n{:?}", req, uuid);

	let ori_uuid = str::replace(&uuid, "hash_", "");
	let action_key = help::uuid_cb_key(&ori_uuid, "action_name");
	let req_key = help::uuid_cb_key(&ori_uuid, "action_req");

	let tmp = help::get_mem_cache(&action_key)?;
	let action_name: &str = bincode::deserialize(&tmp)?;
	let req_bytes = help::get_mem_cache(&req_key)?;

	let rs = match action_name {
		"post_message" => {
			let req: PostMessageRequest = bincode::deserialize(&req_bytes)?;
			let msg_id = channel::post_message_to_db(&req)?;
			json!({
			  "status": true,
			  "msg_id": msg_id,
			})
		}
		"withdraw" => {
			json!({
			  "status": true,
			})
		}
		"notification_add_message" => {
			let req: NotificationAddMessageRequest = bincode::deserialize(&req_bytes)?;
			let msg_id = notification::add_message_to_db(&req)?;
			json!({
			  "status": true,
			  "msg_id": msg_id,
			})
		}
		"login_request" => {
			let req: PrepareLoginRequest = bincode::deserialize(&req_bytes)?;
			// send query 
			let query_uuid = user::login_request_cb(&req)?;

			json!({
				"status": true,
				"need_query": true,
				"query_uuid": query_uuid,
			})
		}
		_ => {
			json!({
			  "status": true
			})
		}
	};

	Ok(rs)
}

pub fn to_query_uuid(uuid: &str) -> String {
	let query_uuid = str::replace(&uuid, "txn_", "");
	let query_uuid = str::replace(&query_uuid, "hash_", "");

	query_uuid.to_string()
}
