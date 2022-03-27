use bincode;
use std::convert::TryInto;
use tea_actor_utility::actor_crypto::{public_key_from_ss58, sha256};
use tea_actor_utility::actor_env::get_system_time;
use tea_actor_utility::actor_enclave::get_my_tea_id;

// use tea_actor_utility::actor_enclave::generate_uuid;
// use base64;
use tea_codec;

use vmh_codec::message::{encode_protobuf, structs_proto::replica};

// use wascc_actor::HandlerResult;
use party_shared::TeapartyTxn;
use wascc_actor::untyped;

use interface::{Account, AuthKey, Balance, Followup, Hash, Ts, Tsid, TxnSerial};
use tea_actor_utility::actor_statemachine::new_txn_serial;

fn get_serial_and_hash_from_txn(txn_bytes: Vec<u8>) -> anyhow::Result<(TxnSerial, Hash)> {
	let txn_serial = new_txn_serial(
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
		txn_bytes.clone(),
	)?;
	let txn_hash: Hash = sha256(bincode::serialize(&txn_serial)?)?
		.as_slice()
		.try_into()
		.expect("wrong length hash");
	Ok((txn_serial, txn_hash))
}

pub fn send_followup_to_replica(followup_bytes: Vec<u8>) -> anyhow::Result<()> {
	// info!("begin to send followup to replica");
	let res = untyped::default()
		.call(
			tea_codec::REPLICA_CAPABILITY_ID,
			tea_codec::ops::replica::OP_REV_FOLLOWUP,
			encode_protobuf(replica::ReceiveFollowup {
				followup: followup_bytes,
			})?,
		)
		.unwrap_or(b"followup faild".to_vec());
	if res.is_empty() {
		info!("[replica provider] OP_REV_FOLLOWUP receive an followup. but res is empty vec");
	} else {
		let tsid: Tsid = bincode::deserialize(&res)?;
		info!("[replica provider] OP_REV_FOLLOWUP => {:?}", tsid);
	}

	Ok(())

	// Err(anyhow::anyhow!("{}", "No followup tsid returned."))
}

pub fn send_tx_to_replica(txn_bytes: Vec<u8>) -> anyhow::Result<()> {
	let (txn_serial, _txn_hash) = get_serial_and_hash_from_txn(txn_bytes)?;
	let req_txn = replica::ReceiveTxn {
		txn_bytes: bincode::serialize(&txn_serial)?,
	};

	// info!("begin to send rev_txn => {:?}", req_txn);
	let res = untyped::default()
		.call(
			tea_codec::REPLICA_CAPABILITY_ID,
			tea_codec::ops::replica::OP_REV_TXN,
			encode_protobuf(req_txn)?,
		)
		.unwrap_or(b"rev_txn faild".to_vec());
	if res.is_empty() {
		info!("[replica provider] OP_REV_TXN receive an existing txn. res is empty vec");
	} else {
		let tsid: Tsid = bincode::deserialize(&res)?;
		info!("[replica provider] OP_REV_TXN => {:?}", tsid);
	}

	Ok(())
}

fn get_current_ts() -> anyhow::Result<Ts> {
	let ts: Ts = get_system_time()?
		.duration_since(std::time::SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_nanos();
	Ok(ts)
}

fn send_actor_hash() -> anyhow::Result<Hash> {
	Ok(get_my_tea_id()?.as_slice().try_into()?)
}

fn execute_tx_with_txn(txn: TeapartyTxn) -> anyhow::Result<()> {
	// step 1, send tx
	let txn_bytes = bincode::serialize(&txn)?;

	send_tx_to_replica(txn_bytes.clone())?;

	let (_, txn_hash) = get_serial_and_hash_from_txn(txn_bytes)?;
	let sent_time = get_current_ts()?;

	// step 2, send followup
	let sender_actor_hash = send_actor_hash()?;
	let req_fu: Followup = Followup {
		ts: sent_time,
		hash: txn_hash,
		sender: sender_actor_hash,
	};
	let fu_bytes = bincode::serialize(&req_fu)?;
	let _ = send_followup_to_replica(fu_bytes)?;

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
