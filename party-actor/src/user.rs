use crate::types::*;
use crate::BINDING_NAME;
use actor_txns::tappstore::TappstoreTxn;
use bincode;
use interface::txn::QuerySerial;
use interface::{AuthKey, Followup, Ts};
use party_shared::TeapartyTxn;
use prost::Message;
use serde_json::json;
use str_utils::*;
use tea_actor_utility::{
	actor_crypto::{
		aes_decrypt, aes_encrypt, generate_aes_key, generate_rsa_keypair, public_key_from_ss58,
		public_key_to_ss58, rsa_decrypt, sign, verify,
	},
	actor_enclave::{get_my_ephemeral_id, get_my_ephemeral_key},
	actor_kvp,
};
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{libp2p, tappstore, tokenstate},
};

use crate::help;
use crate::request::{send_query, send_txn, to_query_uuid};
use crate::utility;

pub fn login_request(req: &LoginRequest) -> anyhow::Result<Vec<u8>> {
	let txn_uuid = req.uuid.to_string();

	let login_request_txn = TappstoreTxn::GenSessionKey {
		token_id: req.tapp_id,
		acct_s58: req.address.to_string(),
		data: base64::decode(&req.data)?,
		signature: base64::decode(&req.signature)?,
		tea_id: utility::send_actor_hash()?.to_vec(),
	};
	let txn_bytes: Vec<u8> = bincode::serialize(&login_request_txn)?;

	send_txn(
		"login_request",
		&txn_uuid,
		bincode::serialize(req)?,
		txn_bytes,
		&tea_codec::ACTOR_PUBKEY_TAPPSTORE.to_string(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn login_request_cb(req: &LoginRequest) -> anyhow::Result<String> {
	let query_bytes = tappstore::tapp_query_request::Msg::CheckUserSessionRequest(
		tappstore::CheckUserSessionRequest {
			account: req.address.to_string(),
			token_id: req.tapp_id,
			tea_id: utility::send_actor_hash()?.to_vec(),
		},
	);
	let query_bytes = tappstore::TappQueryRequest {
		msg: Some(query_bytes),
	};

	let uuid = to_query_uuid(&req.uuid);
	send_query(
		encode_protobuf(query_bytes)?,
		&uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(uuid)
}

pub fn check_auth(tapp_id: &u64, address: &str, auth_b64: &str) -> anyhow::Result<Vec<u8>> {
	let auth_key = help::get_session_key(&tapp_id, &address);

	if !auth_key.is_err() && auth_b64.to_string() == auth_key.unwrap() {
		return Ok(b"is_login".to_vec());
	}

	Err(anyhow::anyhow!("{}", "not_login"))
}

pub fn query_balance(req: &HttpQueryBalanceRequest) -> anyhow::Result<Vec<u8>> {
	check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	info!("begin to query tea balance");

	let auth_key = base64::decode(&req.auth_b64)?;
	let uuid = &req.uuid;
	let req = tappstore::TappQueryRequest {
		msg: Some(tappstore::tapp_query_request::Msg::TeaBalanceRequest(
			tappstore::TeaBalanceRequest {
				account: req.address.to_string(),
				token_id: req.tapp_id,
				auth_key,
			},
		)),
	};

	send_query(
		encode_protobuf(req)?,
		uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn withdraw(req: &WithdrawRequest) -> anyhow::Result<Vec<u8>> {
	check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	let txn = TappstoreTxn::Withdraw {
		token_id: req.tapp_id,
		acct: utility::parse_to_acct(&req.address)?,
		amount: req.amount,
		auth_b64: req.auth_b64.to_string(),
	};
	let txn_bytes: Vec<u8> = bincode::serialize(&txn)?;

	send_txn(
		"withdraw",
		&req.uuid.to_string(),
		bincode::serialize(req)?,
		txn_bytes,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}
