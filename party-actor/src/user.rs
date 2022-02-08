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
use crate::state;

pub fn save_session_key(session_key: String, tapp_id: &u64, address: &str) -> anyhow::Result<()> {
	let key = format!("session_key_{}_{}", tapp_id, address);

	actor_kvp::set(BINDING_NAME, &key, &session_key, 6000 * 120)
		.map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}
pub fn get_session_key(tapp_id: &u64, address: &str) -> anyhow::Result<String> {
	let key = format!("session_key_{}_{}", tapp_id, address);

	let session_key: String =
		actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get session key"))?;

	Ok(session_key)
}

pub fn save_aes_key(_aes_key: Vec<u8>, tapp_id: &u64) -> anyhow::Result<()> {
	let key = format!("aes_key_{}", tapp_id);

	// TODO use real aes_key
	let aes_key: Vec<u8> = vec![8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];
	actor_kvp::set_forever(BINDING_NAME, &key, &aes_key).map_err(|e| anyhow::anyhow!("{}", e))?;

	Ok(())
}
pub fn get_aes_key(tapp_id: &u64) -> anyhow::Result<Vec<u8>> {
	let _key = format!("aes_key_{}", tapp_id);

	// TODO use real
	// let aes_key: Vec<u8> =
	// 	actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get aes key"))?;
	let aes_key: Vec<u8> = vec![8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];

	Ok(aes_key)
}

pub fn prepare_login_request(req: &PrepareLoginRequest) -> anyhow::Result<Vec<u8>> {
	// send to state receive actor
	let uuid = req.uuid.to_string();

	let query_bytes = tappstore::tapp_query_request::Msg::CheckUserSessionRequest(
		tappstore::CheckUserSessionRequest {
			account: req.address.to_string(),
			token_id: req.tapp_id,
			tea_id: help::get_tea_id()?,
		},
	);
	let query_bytes = tappstore::TappQueryRequest {
		msg: Some(query_bytes),
	};
	help::set_mem_cache(
		&help::uuid_cb_key(&uuid, &"check_user_auth"),
		encode_protobuf(query_bytes)?,
	)?;

	let login_request_txn = TappstoreTxn::GenSessionKey {
		token_id: req.tapp_id,
		acct_s58: req.address.to_string(),
		data: base64::decode(&req.data)?,
		signature: base64::decode(&req.signature)?,
		tea_id: help::get_tea_id()?,
	};
	let txn_bytes: Vec<u8> = bincode::serialize(&login_request_txn)?;
	state::execute_tx_with_txn_bytes(txn_bytes, uuid, tea_codec::ACTOR_PUBKEY_TAPPSTORE.into())?;

	Ok(b"ok".to_vec())
}

pub fn libp2p_msg_cb(body: &tokenstate::StateReceiverResponse) -> anyhow::Result<bool> {
	let uuid = &body.uuid;

	if uuid.starts_with_ignore_ascii_case("check_user") {
		match &body.msg {
			Some(tokenstate::state_receiver_response::Msg::GeneralQueryResponse(r)) => {
				let query_res = tappstore::TappQueryResponse::decode(r.data.as_slice())?;

				libp2p_msg_cb_handler(&query_res)?;

				return Ok(true);
			}
			_ => warn!("unknown state receiver response: {:?}", body.msg),
		}
	}

	Ok(false)
}

fn libp2p_msg_cb_handler(res: &tappstore::TappQueryResponse) -> anyhow::Result<()> {
	match &res.msg {
		Some(tappstore::tapp_query_response::Msg::CheckUserSessionResponse(r)) => {
			let aes_key = &r.aes_key;
			let auth_key = &r.auth_key;

			let auth_b64 = base64::encode(auth_key);
			info!("save auth_b64 => {:?}", auth_b64);
			info!("save aes_key => {:?}", aes_key);

			save_session_key(auth_b64, &r.token_id, &r.account)?;
			save_aes_key(aes_key.to_vec(), &r.token_id)?;
		}
		_ => warn!("unknown tapp_query_response: {:?}", res.msg),
	}

	Ok(())
}

pub fn check_user_query_uuid(uuid: &str) -> String {
	format!("check_user_{}", uuid)
}

pub fn check_auth(tapp_id: &u64, address: &str, auth_b64: &str) -> anyhow::Result<Vec<u8>> {
	let auth_key = get_session_key(&tapp_id, &address);

	if !auth_key.is_err() && auth_b64.to_string() == auth_key.unwrap() {
		return Ok(b"is_login".to_vec());
	}

	Err(anyhow::anyhow!("{}", "not_login"))
}

pub fn update_tapp_profile(req: &TappProfileRequest) -> anyhow::Result<Vec<u8>> {
	// let a = [0u8; 32];
	// let b = [255u8; 32];

	// info!("aaaa => {:?}", public_key_to_ss58(&a));
	// info!("bbbb => {:?}", public_key_to_ss58(&b));

	check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	info!("state begin to update profile");
	let txn = TeapartyTxn::UpdateProfile {
		acct: state::parse_to_acct(&req.address)?,
		token_id: req.tapp_id,
		auth_b64: req.auth_b64.to_string(),
		post_message_fee: req.post_message_fee,
	};
	let txn_bytes = bincode::serialize(&txn)?;
	state::execute_tx_with_txn_bytes(
		txn_bytes,
		req.uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;
	info!("state update profile success");

	Ok(b"ok".to_vec())
}

pub fn query_balance(req: &HttpQueryBalanceRequest) -> anyhow::Result<Vec<u8>> {
	// check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	info!("begin to query tea balance");

	// let auth_key = base64::decode(&req.auth_b64)?;
	let uuid = &req.uuid;
	let req = tappstore::TappQueryRequest {
		msg: Some(tappstore::tapp_query_request::Msg::TeaBalanceRequest(
			tappstore::TeaBalanceRequest {
				account: req.address.to_string(),
				token_id: req.tapp_id,
				auth_key: [0;8].to_vec(),
			},
		)),
	};

	state::send_query_via_p2p(
		encode_protobuf(req)?,
		uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn withdraw(req: &WithdrawRequest) -> anyhow::Result<Vec<u8>> {
	check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	info!("start to withdraw action...");

	let txn = TappstoreTxn::Withdraw {
		token_id: req.tapp_id,
		acct: state::parse_to_acct(&req.address)?,
		amount: req.amount,
		auth_b64: req.auth_b64.to_string(),
	};
	let txn_bytes: Vec<u8> = bincode::serialize(&txn)?;
	state::execute_tx_with_txn_bytes(
		txn_bytes,
		req.uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}
