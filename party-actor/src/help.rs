use interface::Tsid;
use prost::Message;

use tea_actor_utility::{
	actor_enclave::{generate_uuid, get_my_tea_id},
	actor_env::{get_system_time, time_since},
	actor_kvp,
	actor_layer1::general_remote_request,
	actor_libp2p,
};

use base64;
use serde_json::json;
use tea_codec;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{layer1, libp2p, tappstore, tokenstate, orbitdb},
};
use wascc_actor::untyped;

use crate::BINDING_NAME;

pub fn p2p_send_to_receive_actor(msg: Vec<u8>) -> anyhow::Result<()> {
	// let my_tea_id = get_my_tea_id()?;
	let a_nodes = get_all_active_a_nodes()?;

	info!("all A nodes => {:?}", a_nodes);

	if a_nodes.len() < 1 {
		return Err(anyhow::anyhow!("{}", "No active A nodes."));
	}

	let target_conn_id = conn_id_by_tea_id(a_nodes[0].clone())?;
	info!("target conn id => {:?}", target_conn_id);

	// TOOD send to at least 2 A node.
	for node in a_nodes {
		info!("loop for all A nodes...");
		let target_conn_id = conn_id_by_tea_id(node.clone())?;
		info!("target conn id => {:?}", target_conn_id);

		let target_key = tea_codec::ACTOR_PUBKEY_STATE_RECEIVER.to_string();
		let target_type = libp2p::TargetType::Actor as i32;

		// TODO, convert to send

		info!("p2p send msg start...");
		actor_libp2p::send_message(
			target_conn_id,
			libp2p::RuntimeAddress {
				target_key,
				target_type,
				target_action: "libp2p.state-receiver".to_string(),
			},
			None,
			msg.clone(),
		)?;
	}

	info!("p2p send msg finish...");

	Ok(())
}

pub fn get_all_active_a_nodes() -> anyhow::Result<Vec<Vec<u8>>> {
	let res_buf = general_remote_request(layer1::Layer1Outbound {
		msg: Some(layer1::layer1_outbound::Msg::ListMiningCmlsRequest(
			layer1::ListMiningCmlsRequest {},
		)),
	})?;
	let res = layer1::ListMiningCmlsResponse::decode(res_buf.as_slice())?;

	let current_a_miners: Vec<Vec<u8>> = res
		.mining_cmls
		.iter()
		.filter(|info| info.cml_type.eq("A") && info.miner_status.eq("Active"))
		.map(|info| info.tea_id.clone())
		.collect();
	Ok(current_a_miners)
}

pub fn conn_id_by_tea_id(tea_id: Vec<u8>) -> anyhow::Result<String> {
	let res_buf = general_remote_request(layer1::Layer1Outbound {
		msg: Some(layer1::layer1_outbound::Msg::GetConnIdRequest(
			layer1::GetConnIdRequest { tea_id },
		)),
	})?;
	let res = layer1::GetConnIdResponse::decode(res_buf.as_slice())?;
	Ok(res.conn_id)
}

pub fn set_mem_cache(key: &str, val: Vec<u8>) -> anyhow::Result<()> {
	actor_kvp::set(BINDING_NAME, &key, &val, 600).map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}

pub fn get_mem_cache(key: &str) -> anyhow::Result<Vec<u8>> {
	let rs: Vec<u8> = actor_kvp::get(BINDING_NAME, &key)?
		.ok_or(anyhow::anyhow!("failed to get value with {}", key))?;

	Ok(rs)
}

pub fn del_mem_cache(key: &str) -> anyhow::Result<()> {
	actor_kvp::del(BINDING_NAME, &key)?;
	Ok(())
}

pub fn to_json_response(key: &str) -> anyhow::Result<serde_json::Value> {
	let value = get_mem_cache(key)?;
	
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
	}
	else if let Ok(res) = orbitdb::OrbitBbsResponse::decode(value.as_slice()) {
		return Ok(json!({
			// "data": res.data.to_string(),
			"status": "ok".to_string()
		}));
	}

	Ok(json!({"error": format!("unknown value for key : {}", key)}))	
}

fn parse_tappstore_response(data: &[u8], uuid: &str) -> anyhow::Result<serde_json::Value> {
	let tapp_query_response = tappstore::TappQueryResponse::decode(data)?;
	info!("1111 => {:?}", tapp_query_response);
	let rtn = match tapp_query_response.msg {
		Some(tappstore::tapp_query_response::Msg::TeaBalanceResponse(balance_res)) => {
			json!({
			  "balance": u128_from_le_buffer(&balance_res.balance)?.to_string(),
			  "ts": u128_from_le_buffer(&balance_res.ts)?.to_string(),
			  "uuid": uuid.to_string(),
			})
		}
		Some(tappstore::tapp_query_response::Msg::FindExecutedTxnResponse(r)) => {
			if let Some(res) = r.executed_txn {
				json!({
					// "tsid": hex::encode(&res.tsid),
					"status": true,
				})
			} else {
				json!({
					"status": false,
				})
			}
		}
		Some(tappstore::tapp_query_response::Msg::CheckUserSessionResponse(r)) => {
			let auth_key = &r.auth_key;

			json!({
				"auth_key": base64::encode(auth_key),
			})
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

pub fn uuid_cb_key(uuid: &str, stype: &str) -> String {
	let rs = format!("{}_msg_{}", stype, uuid);
	rs.to_string()
}
pub fn cb_key_to_uuid(key: &str, stype: &str) -> String {
	let ss = format!("{}_msg_", stype);
	let rs = str::replace(key, &ss, "");
	rs.to_string()
}

pub fn get_tea_id() -> anyhow::Result<Vec<u8>> {
	Ok([0; 32].to_vec())
}
