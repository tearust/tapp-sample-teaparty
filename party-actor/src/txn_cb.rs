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

use crate::api;
use crate::help;
use crate::message;
use crate::notification;
use crate::types::*;
use crate::user;
use crate::utility::uuid_cb_key;

pub fn txn_callback(
	req: replica::FindExecutedTxnResponse,
	uuid: &str,
) -> anyhow::Result<serde_json::Value> {
	info!("txn_callback => {:?}\n{:?}", req, uuid);

	let ori_uuid = str::replace(&uuid, "hash_", "");
	let action_key = uuid_cb_key(&ori_uuid, "action_name");
	let req_key = uuid_cb_key(&ori_uuid, "action_req");

	let tmp = help::get_mem_cache(&action_key)?;
	let action_name: &str = bincode::deserialize(&tmp)?;
	let req_bytes = help::get_mem_cache(&req_key)?;

	let rs = match action_name {
		"post_message" => {
			let req: PostMessageRequest = bincode::deserialize(&req_bytes)?;
			let msg_id = message::post_message_to_db(&req)?;
			json!({
			  "status": true,
			  "msg_id": msg_id,
			})
		}
		"extend_message" => {
			let req: ExtendMessageRequest = bincode::deserialize(&req_bytes)?;
			message::extend_message_to_db(&req)?;

			json!({
				"status": true,
			})
		}
		"delete_message" => {
			let req: DeleteMessageRequest = bincode::deserialize(&req_bytes)?;
			message::delete_message_to_db(&req)?;

			json!({
				"status": true,
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
			let req: LoginRequest = bincode::deserialize(&req_bytes)?;
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
