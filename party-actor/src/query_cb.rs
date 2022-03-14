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

use crate::help;
use crate::txn_cb::txn_callback;
use crate::types::BINDING_NAME;
use crate::user;

pub fn query_callback(key: &str) -> anyhow::Result<serde_json::Value> {
	let value = help::get_mem_cache(key)?;

	if let Ok(res) = tokenstate::StateReceiverResponse::decode(value.as_slice()) {
		let rtn = match res.msg.as_ref() {
			Some(tokenstate::state_receiver_response::Msg::GeneralQueryResponse(r)) => {
				parse_tappstore_response(&r.data, &res.uuid)?
			}
			Some(tokenstate::state_receiver_response::Msg::CommandFollowupResponse(cf_res)) => {
				json!({
					"ts": u128_from_le_buffer(&cf_res.ts)?.to_string(),
					"hash": hex::encode(&cf_res.hash),
					"sender": hex::encode(&cf_res.sender),
					"uuid": res.uuid.clone(),
				})
			}
			Some(tokenstate::state_receiver_response::Msg::DirectResponse(_)) => {
				json!({"status": "pending"})
			}
			_ => json!({ "error": format!("unknown response: {:?}", res) }),
		};
		return Ok(rtn);
	} else if let Ok(_res) = orbitdb::OrbitBbsResponse::decode(value.as_slice()) {
		return Ok(json!({
			// "data": res.data.to_string(),
			"status": "ok".to_string()
		}));
	}

	Ok(json!({
		"error": format!("unknown value for key : {}", key)
	}))
}

fn parse_tappstore_response(data: &[u8], uuid: &str) -> anyhow::Result<serde_json::Value> {
	let tapp_query_response = tappstore::TappQueryResponse::decode(data)?;
	info!("tapp_query_response => {:?}", tapp_query_response);
	let rtn = match tapp_query_response.msg {
		None => {
			json!({
				"error": "none",
			})
		}
		Some(tappstore::tapp_query_response::Msg::TeaBalanceResponse(balance_res)) => {
			json!({
			  "balance": u128_from_le_buffer(&balance_res.balance)?.to_string(),
			  "ts": u128_from_le_buffer(&balance_res.ts)?.to_string(),
			  "uuid": uuid.to_string(),
			})
		}
		Some(tappstore::tapp_query_response::Msg::FindExecutedTxnResponse(r)) => {
			info!("FindExecutedTxnResponse => {:?}", r);

			if r.clone().success == true {
				if r.clone().executed_txn.is_some() {
					info!("Txn hash return success. go to next step...");
					txn_callback(r.clone(), &uuid)?
				} else {
					info!("Txn hash no error. but not success. wait for next loop...");
					json!({
						"status": false,
						"error": "wait",
					})
				}
			} else {
				json!({
					"status": false,
					"error": &r.error_msg,
				})
			}
		}
		Some(tappstore::tapp_query_response::Msg::CheckUserSessionResponse(r)) => {
			let aes_key = &r.aes_key;
			let auth_key = &r.auth_key;

			let auth_b64 = base64::encode(auth_key);
			info!("save auth_b64 => {:?}", auth_b64);
			info!("save aes_key => {:?}", aes_key);

			help::save_session_key(auth_b64, &r.token_id, &r.account)?;
			help::save_aes_key(aes_key.to_vec(), &r.token_id)?;
			let auth_key = &r.auth_key;

			json!({
				"auth_key": base64::encode(auth_key),
			})
		}
		Some(tappstore::tapp_query_response::Msg::GetConsumeAccountPubkeyResponse(r)) => {
			let address = &r.address;
			json!({
				"address": address,
			})
		}
		Some(tappstore::tapp_query_response::Msg::GetTappstoreAccountPubkeyResponse(r)) => {
			let address = &r.address;
			json!({
				"address": address,
			})
		}
		Some(tappstore::tapp_query_response::Msg::CommonSqlQueryResponse(r)) => {
			if !r.err.is_empty() {
				error!("sql error: {}", &r.err);
				json!({
					"status": false,
					"error": &r.err,
				})
			} else {
				let result_payload: Vec<Payload> = bincode::deserialize(&r.data)?;
				info!(
					"parse_tappstore_response, deser result_payload is {:?}",
					&result_payload
				);
				let mut rows: Vec<String> = Vec::new();
				for p in result_payload {
					let line = match p {
						Payload::Select { labels: _, rows } => {
							format!("{:?}", &rows)
						}
						_ => format!("Query error: {:?}", p),
					};
					rows.push(line);
				}
				info!("rows {:?}", &rows);
				json!({ "sql_query_result": rows })
			}
		}
		_ => json!({ "error": format!("unknown tappstore response: {:?}", tapp_query_response) }),
	};
	Ok(rtn)
}

fn u128_from_le_buffer(data: &[u8]) -> anyhow::Result<u128> {
	const U128_LENGTH: usize = 16;

	if data.len() < U128_LENGTH {
		return Err(anyhow::anyhow!("u128 length should be {}", U128_LENGTH));
	}

	let mut u128_buf = [0u8; U128_LENGTH];
	u128_buf.copy_from_slice(&data[0..U128_LENGTH]);
	Ok(u128::from_le_bytes(u128_buf))
}
