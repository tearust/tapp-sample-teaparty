
use bincode;
use std::convert::TryInto;
use tea_actor_utility::actor_env::{get_system_time, };
use tea_actor_utility::actor_crypto::{ sha256, public_key_from_ss58};

use tea_actor_utility::actor_enclave::generate_uuid;
use base64;
use tea_codec;
use serde_json::{json};

use wascc_actor::HandlerResult;
use party_shared::{TeapartyTxn};

use interface::{
  Hash, TxnSerial, Followup, Ts,
  Account, Balance,
};
use tea_actor_utility::actor_rpc::layer1::execute_http_request_ex;


fn get_hash_from_txn(
	txn_bytes: Vec<u8>
) -> anyhow::Result<Hash> {
	let txn_serial = TxnSerial{
		actor_name: tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
		bytes: txn_bytes.clone(),
	};
	let txn_hash: Hash = sha256(bincode::serialize(&txn_serial)?)?
		.as_slice()
		.try_into()
		.expect("wrong length hash");
	Ok(txn_hash)
}

fn send_tx_via_http(
	txn: TeapartyTxn,
) -> anyhow::Result<(Ts, Hash)> {
	let hostname = "http://host.docker.internal:8000";
	let url = format!("{}/tapp/state_command", hostname);

	let txn_bytes = bincode::serialize(&txn)?;
	let txn_b64 = base64::encode(txn_bytes.clone());
	let txn_hash = get_hash_from_txn(txn_bytes)?;

	info!("txn_b64 => {:?}", txn_b64);
	let payload = Some(txn_b64.as_bytes().to_vec());

	let uuid = generate_uuid()?;

	
	let _res_str = execute_http_request_ex(url.as_str(), uuid.to_string(), vec![], "POST".into(), payload, Some(3000)).unwrap_or("failed here.".to_string());

	let sent_time = get_current_ts()?;
	Ok((sent_time, txn_hash))
}

fn get_current_ts() -> anyhow::Result<Ts> {
	let ts: Ts = get_system_time()?
		.duration_since(std::time::SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_nanos();
	Ok(ts)
}

fn send_actor_hash() -> Hash {
	[0u8;32]
}

fn send_followup_via_http(
	fu: Followup,
) -> anyhow::Result<()> {
	let hostname = "http://host.docker.internal:8000";
	let url = format!("{}/tapp/state_followup", hostname);

	let fu_bytes = bincode::serialize(&fu)?;
	let fu_b64 = base64::encode(fu_bytes);

	let payload = Some(fu_b64.as_bytes().to_vec());

	let uuid = generate_uuid()?;
	let res_str = execute_http_request_ex(url.as_str(), uuid.to_string(), vec![], "POST".into(), payload, None)?;

	info!("followup response => {:?}", res_str);

	Ok(())
}

fn send_query_via_http(
	query: Vec<u8>
) -> anyhow::Result<()> {
	let hostname = "http://host.docker.internal:8000";
	let url = format!("{}/tapp/state_query", hostname);

	let payload = Some(query);

	let uuid = generate_uuid()?;

	info!("send http query");
	let _res_str = execute_http_request_ex(url.as_str(), uuid.to_string(), vec![], "POST".into(), payload, None)?;

	Ok(())
}

fn execute_tx_with_txn(
	txn: TeapartyTxn
) -> anyhow::Result<()> {
	// step 1, send tx
	let (sent_time, txn_hash) = send_tx_via_http(txn)?;
	// step 2, send followup
	let sender_actor_hash = send_actor_hash();
	let req_fu: Followup = Followup{
		ts: vec!(sent_time),
		hash: txn_hash,
		sender: sender_actor_hash,
	};

	send_followup_via_http(req_fu)?;

	Ok(())
}

pub(crate) fn topup(
  acct: Account,
  amt: Balance
) -> anyhow::Result<()> {
  info!("begin to topup");

  let txn = TeapartyTxn::Topup{
    acct,
    amt,
  };

  execute_tx_with_txn(txn)?;

  info!("topup success!");

  Ok(())
}




pub(crate) fn query_tea_balance(acct_str: &str) -> anyhow::Result<Vec<u8>> {

	info!("begin to query tea balance");
	let query = json!({
		"msg_type": "tea_balance".to_string(),
		"acct": acct_str, 
	});
	
	let query_bytes = serde_json::to_vec(&query).unwrap();

	send_query_via_http(query_bytes)?;

	Ok(b"100".to_vec())
}


pub(crate) fn parse_to_acct(ss58_address: &str) -> anyhow::Result<Account> {
	let acct = public_key_from_ss58(&ss58_address)?;
	if acct.len() != 32 {
		return Err(anyhow::anyhow!("{}", "Invalid ss58 account."));
	}
	let acct: Account = acct.try_into().unwrap();

	Ok(acct)
}