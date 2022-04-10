use bincode;
use interface::{Followup, Hash, QuerySerial, Ts, TxnSerial};
use prost::Message;
use std::convert::TryInto;
use tea_actor_utility::{
	actor_crypto::{sha256},
	actor_enclave::get_my_tea_id,
	actor_env::{get_system_time},
	actor_layer1::general_remote_request,
	actor_libp2p,
	actor_statemachine::new_txn_serial,
};
use tea_codec;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{layer1, libp2p, tokenstate},
};

const AT_LEAST_A_NODES_TO_SEND: usize = 3;

fn get_hash_from_txn(
	txn_bytes: Vec<u8>,
	to_actor_name: String,
) -> anyhow::Result<(Hash, TxnSerial)> {
	let txn_serial = new_txn_serial(to_actor_name, txn_bytes.clone())?;
	let txn_hash: Hash = sha256(bincode::serialize(&txn_serial)?)?
		.as_slice()
		.try_into()
		.expect("wrong length hash");
	Ok((txn_hash, txn_serial))
}

fn get_all_active_a_nodes() -> anyhow::Result<Vec<Vec<u8>>> {
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

fn p2p_send_to_receive_actor(msg: Vec<u8>) -> anyhow::Result<()> {
	let a_nodes = get_all_active_a_nodes()?;

	info!("all A nodes => {:?}", a_nodes);

	let mut len: usize = a_nodes.len();
	if a_nodes.len() < 1 {
		return Err(anyhow::anyhow!("{}", "No active A nodes."));
	} else if a_nodes.len() == 1 {
		warn!("Only 1 node to send, not safe.");
	} else if a_nodes.len() >= AT_LEAST_A_NODES_TO_SEND {
		info!(
			"Enough node to send. global => {}, require => {}",
			a_nodes.len(),
			AT_LEAST_A_NODES_TO_SEND
		);
		len = AT_LEAST_A_NODES_TO_SEND;
	}

	for node in &a_nodes[..len] {
		let target_conn_id = conn_id_by_tea_id(node.clone())?;
		info!("target conn id => {:?}", target_conn_id);

		let target_key = tea_codec::ACTOR_PUBKEY_STATE_RECEIVER.to_string();
		let target_type = libp2p::TargetType::Actor as i32;

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

fn conn_id_by_tea_id(tea_id: Vec<u8>) -> anyhow::Result<String> {
	let res_buf = general_remote_request(layer1::Layer1Outbound {
		msg: Some(layer1::layer1_outbound::Msg::GetConnIdRequest(
			layer1::GetConnIdRequest { tea_id },
		)),
	})?;
	let res = layer1::GetConnIdResponse::decode(res_buf.as_slice())?;
	Ok(res.conn_id)
}

fn get_current_ts() -> anyhow::Result<Ts> {
	let ts: Ts = get_system_time()?
		.duration_since(std::time::SystemTime::UNIX_EPOCH)?
		.as_nanos();
	Ok(ts)
}

pub fn send_actor_hash() -> anyhow::Result<Hash> {
	Ok(get_my_tea_id()?.as_slice().try_into()?)
}

pub fn send_tx_via_p2p(
	txn_bytes: Vec<u8>,
	uuid: String,
	to_actor_name: String,
) -> anyhow::Result<(Ts, Hash)> {
	let (txn_hash, txn_serial) = get_hash_from_txn(txn_bytes, to_actor_name.clone())?;

	let payload = encode_protobuf(tokenstate::StateReceiverMessage {
		uuid,
		msg: Some(tokenstate::state_receiver_message::Msg::StateCommand(
			tokenstate::StateCommand {
				data: txn_serial.bytes().to_vec(),
				target: txn_serial.actor_name().to_string(),
				nonce: txn_serial.nonce(),
			},
		)),
	})?;
	info!("txn payload => {:?}", payload);

	p2p_send_to_receive_actor(payload)?;

	let sent_time = get_current_ts()?;
	Ok((sent_time, txn_hash))
}

pub fn send_followup_via_p2p(fu: Followup, uuid: String) -> anyhow::Result<()> {
	let fu_bytes = bincode::serialize(&fu)?;

	let payload = encode_protobuf(tokenstate::StateReceiverMessage {
		uuid,
		msg: Some(tokenstate::state_receiver_message::Msg::StateFollowup(
			tokenstate::StateFollowup { data: fu_bytes },
		)),
	})?;
	info!("followup payload => {:?}", payload);

	p2p_send_to_receive_actor(payload)?;

	Ok(())
}

pub fn p2p_send_query(
	query_bytes: Vec<u8>,
	uuid: &str,
	to_actor_name: String,
) -> anyhow::Result<()> {
	let serial = QuerySerial {
		actor_name: to_actor_name.clone(),
		bytes: query_bytes,
	};
	let payload = encode_protobuf(tokenstate::StateReceiverMessage {
		uuid: uuid.to_string(),
		msg: Some(tokenstate::state_receiver_message::Msg::StateQuery(
			tokenstate::StateQuery {
				data: bincode::serialize(&serial)?,
				target: to_actor_name,
			},
		)),
	})?;
	info!("query payload => {:?}", payload);

	p2p_send_to_receive_actor(payload)?;

	Ok(())
}

pub fn p2p_send_txn(txn_bytes: Vec<u8>, uuid: String, to_actor_name: String) -> anyhow::Result<()> {
	// step 1, send tx
	let (sent_time, txn_hash) = send_tx_via_p2p(txn_bytes, uuid.clone(), to_actor_name)?;
	// step 2, send followup
	let sender_actor_hash = send_actor_hash()?;
	let req_fu: Followup = Followup {
		ts: sent_time,
		hash: txn_hash,
		sender: sender_actor_hash,
	};

	send_followup_via_p2p(req_fu, uuid.clone())?;

	Ok(())
}
