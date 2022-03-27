use actor_txns::tappstore::TappstoreTxn;
use bincode;
use std::str::FromStr;
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
	structs_proto::{libp2p, replica, tappstore, tokenstate},
};

use crate::help;
use crate::message;
use crate::notification;
use crate::query_cb::query_callback;
use crate::request::{send_query, send_txn};
use crate::types::*;
use crate::user;
use crate::utility::{self, p2p_send_txn};

pub fn query_result(req: &HttpQueryResultWithUuid) -> anyhow::Result<serde_json::Value> {
	query_callback(&req.uuid)
}

pub fn query_txn_hash_result(req: &QueryHashRequest) -> anyhow::Result<Vec<u8>> {
	info!("begin to query hash result...");

	let txn_hash = hex::decode(req.hash.clone())?;
	let uuid = &req.uuid;
	let ts = bincode::serialize(&u128::from_str(&req.ts)?)?;

	let req = tappstore::TappQueryRequest {
		msg: Some(tappstore::tapp_query_request::Msg::FindExecutedTxnRequest(
			replica::FindExecutedTxnRequest { txn_hash, ts, },
		)),
	};

	send_query(
		encode_protobuf(req)?,
		&uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn query_tapp_account(req: &QueryTappAccountRequest) -> anyhow::Result<Vec<u8>> {
	info!("begin to query tapp account...");

	let tapp_id = &req.tapp_id;
	let uuid = req.uuid.to_string();
	let req = tappstore::TappQueryRequest {
		msg: Some(
			tappstore::tapp_query_request::Msg::GetConsumeAccountPubkeyRequest(
				tappstore::GetConsumeAccountPubkeyRequest { token_id: *tapp_id },
			),
		),
	};

	send_query(
		encode_protobuf(req)?,
		&uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}

pub fn query_tappstore_account(req: &QueryTappStoreAccountRequest) -> anyhow::Result<Vec<u8>> {
	info!("begin to query tappstore account...");

	let uuid = &req.uuid;
	let req = tappstore::TappQueryRequest {
		msg: Some(
			tappstore::tapp_query_request::Msg::GetTappstoreAccountPubkeyRequest(
				tappstore::GetTappstoreAccountPubkeyRequest {},
			),
		),
	};

	send_query(
		encode_protobuf(req)?,
		&uuid,
		tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	)?;

	Ok(b"ok".to_vec())
}

// user api
pub fn login_request(req: &LoginRequest) -> anyhow::Result<Vec<u8>> {
	user::login_request(req)
}

pub fn query_balance(req: &HttpQueryBalanceRequest) -> anyhow::Result<Vec<u8>> {
	user::query_balance(req)
}

pub fn withdraw(req: &WithdrawRequest) -> anyhow::Result<Vec<u8>> {
	user::withdraw(req)
}

pub fn logout(_req: &LogoutRequest) -> anyhow::Result<Vec<u8>> {
	// TODO
	Ok(b"ok".to_vec())
}

// message api
pub fn post_message(req: &PostMessageRequest) -> anyhow::Result<Vec<u8>> {
	message::post_message(req)
}

pub fn post_free_message(req: &PostMessageRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;
	message::post_message_to_db(&req)?;
	Ok(b"ok".to_vec())
}

pub fn load_message_list(req: &LoadMessageRequest) -> anyhow::Result<Vec<u8>> {
	message::load_message_list(req)
}

pub fn extend_message(req: &ExtendMessageRequest) -> anyhow::Result<Vec<u8>> {
	message::extend_message(req)
}

pub fn delete_message(req: &DeleteMessageRequest) -> anyhow::Result<Vec<u8>> {
	message::delete_message(req)
}

// notification api
pub fn notification_add_message(req: &NotificationAddMessageRequest) -> anyhow::Result<Vec<u8>> {
	notification::add_message(req)
}
pub fn notification_get_message_list(
	req: &NotificationGetMessageRequest,
) -> anyhow::Result<Vec<u8>> {
	notification::get_message_list(req)
}

// profile
pub fn update_tapp_profile(req: &TappProfileRequest) -> anyhow::Result<Vec<u8>> {
	user::check_auth(&req.tapp_id, &req.address, &req.auth_b64)?;

	info!("state begin to update profile");
	let txn = TeapartyTxn::UpdateProfile {
		acct: utility::parse_to_acct(&req.address)?,
		token_id: req.tapp_id,
		auth_b64: req.auth_b64.to_string(),
		post_message_fee: req.post_message_fee,
	};
	let txn_bytes = bincode::serialize(&txn)?;
	p2p_send_txn(
		txn_bytes,
		req.uuid.to_string(),
		tea_codec::ACTOR_PUBKEY_PARTY_CONTRACT.to_string(),
	)?;
	info!("state update profile success");

	Ok(b"ok".to_vec())
}

// test
pub fn send_sql_for_test(req: &TestForSqlRequest) -> anyhow::Result<Vec<u8>> {
	let uuid = &req.uuid;

	if req.is_txn {
		info!("start to send sql txn...req is {:?}", req);
		let txn = TappstoreTxn::SqlTest {
			token_id: req.tapp_id,
			sql: req.sql.clone(),
		};
		let txn_bytes: Vec<u8> = bincode::serialize(&txn)?;
		p2p_send_txn(
			txn_bytes,
			uuid.clone(),
			tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
		)?;
	} else {
		info!("start to send sql query...req is {:?}", &req);

		let req = tappstore::TappQueryRequest {
			msg: Some(tappstore::tapp_query_request::Msg::CommonSqlQueryRequest(
				tappstore::CommonSqlQueryRequest {
					token_id: req.tapp_id,
					sql: req.sql.clone(),
				},
			)),
		};

		send_query(
			encode_protobuf(req)?,
			&uuid,
			tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
		)?;
	}

	Ok(b"ok".to_vec())
}

pub fn send_test_for_comsume_dividend(req: &TestForComsumeDividend) -> anyhow::Result<Vec<u8>> {
	let _uuid = &req.uuid;

	info!("start to send test consume dividend txn...");
	// TODO
	// let txn = TappstoreTxn::ConsumeToDividend {
	// 	token_id: req.tapp_id,
	// };
	// let txn_bytes: Vec<u8> = bincode::serialize(&txn)?;
	// p2p_send_txn(
	// 	txn_bytes,
	// 	uuid.clone(),
	// 	tea_codec::ACTOR_PUBKEY_TAPPSTORE.into(),
	// )?;

	Ok(b"ok".to_vec())
}
