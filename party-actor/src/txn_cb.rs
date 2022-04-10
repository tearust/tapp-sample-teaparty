use serde_json::json;
use vmh_codec::message::{
	structs_proto::{replica,},
};

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
