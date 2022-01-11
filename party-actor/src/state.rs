#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
use bincode;
use interface::txn::QuerySerial;
use prost::Message;
use std::convert::TryInto;
use tea_actor_utility::actor_crypto::{public_key_from_ss58, sha256};
use tea_actor_utility::actor_env::get_system_time;

use base64;
use serde_json::json;
use tea_actor_utility::actor_enclave::{generate_uuid, get_my_tea_id};
use tea_codec;

use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{tappstore, tokenstate},
};

use party_shared::TeapartyTxn;
use wascc_actor::HandlerResult;

use interface::{Account, AuthKey, Balance, Followup, Hash, Ts, TxnSerial};

use crate::help;
use crate::types;

fn get_hash_from_txn(txn_bytes: Vec<u8>, to_actor_name: String) -> anyhow::Result<Hash> {
	let txn_serial = TxnSerial {
		actor_name: to_actor_name,
		bytes: txn_bytes.clone(),
	};
	let txn_hash: Hash = sha256(bincode::serialize(&txn_serial)?)?
		.as_slice()
		.try_into()
		.expect("wrong length hash");
	Ok(txn_hash)
}

pub fn send_tx_via_p2p(
	txn_bytes: Vec<u8>,
	uuid: String,
	to_actor_name: String,
) -> anyhow::Result<(Ts, Hash)> {
	let txn_hash = get_hash_from_txn(txn_bytes.clone(), to_actor_name.clone())?;

	let payload = encode_protobuf(tokenstate::StateReceiverMessage {
		uuid,
		msg: Some(tokenstate::state_receiver_message::Msg::StateCommand(
			tokenstate::StateCommand {
				data: txn_bytes,
				target: to_actor_name,
			},
		)),
	})?;
	info!("txn payload => {:?}", payload);

	help::p2p_send_to_receive_actor(payload)?;

	let sent_time = get_current_ts()?;
	Ok((sent_time, txn_hash))
}

pub fn send_query_via_p2p(
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

	help::p2p_send_to_receive_actor(payload)?;

	Ok(())
}

pub fn get_current_ts() -> anyhow::Result<Ts> {
	let ts: Ts = get_system_time()?
		.duration_since(std::time::SystemTime::UNIX_EPOCH)?
		.as_nanos();
	Ok(ts)
}

pub fn send_actor_hash() -> anyhow::Result<Hash> {
	Ok(get_my_tea_id()?.as_slice().try_into()?)
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

	help::p2p_send_to_receive_actor(payload)?;

	// info!("followup response => {:?}", res_str);

	Ok(())
}

pub fn execute_tx_with_txn_bytes(
	txn_bytes: Vec<u8>,
	uuid: String,
	to_actor_name: String,
) -> anyhow::Result<()> {
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

pub fn post_message(acct: &str, ttl: u64, uuid: &str, auth: AuthKey) -> anyhow::Result<()> {
	info!("state begin to post_message");
	let txn = TeapartyTxn::PostMessage {
		from: parse_to_acct(acct)?,
		ttl,
		uuid: uuid.to_string(),
		auth,
	};
	let txn_bytes = bincode::serialize(&txn)?;
	execute_tx_with_txn_bytes(
		txn_bytes,
		uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;
	info!("state post message success");

	Ok(())
}

pub(crate) fn parse_to_acct(ss58_address: &str) -> anyhow::Result<Account> {
	let acct = public_key_from_ss58(&ss58_address)?;
	if acct.len() != 32 {
		return Err(anyhow::anyhow!("{}", "Invalid ss58 account."));
	}
	let acct: Account = acct.try_into().unwrap();

	Ok(acct)
}
