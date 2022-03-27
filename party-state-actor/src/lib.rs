#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]
#[macro_use]
extern crate log;
use interface::{Account, AuthKey, Balance, Tsid, GOD_MODE_AUTH_KEY, TOKEN_ID_TEA};
use party_shared::TeapartyTxn;
use prost::Message;
use std::convert::TryInto;
use tea_actor_utility::actor_crypto::sha256;
use tea_actor_utility::actor_replica::report_txn_error;
use tea_actor_utility::{
	action::post_intercom,
	actor_crypto::{public_from_private_key, public_key_from_ss58, public_key_to_ss58},
	actor_enclave::get_my_tea_id,
	actor_env::{get_env_var, get_system_time},
	actor_layer1::{fetch_miner_info_remotely, register_layer1_event},
	actor_statemachine::{self, query_auth_ops_bytes},
};
use tea_codec::ops::crypto::KEY_TYPE_SR25519;
use tea_codec::{ACTOR_PUBKEY_PARTY_CONTRACT, ACTOR_PUBKEY_TOKENSTATE_SERVICE};
use token_state::token_context::TokenContext;
use vmh_codec::message::structs_proto::tokenstate::*;
use vmh_codec::message::structs_proto::{tappstore, tokenstate};
use vmh_codec::message::{encode_protobuf, layer1::MinerClass, structs_proto::layer1};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;
use wascc_actor::HandlerResult;

const BINDING_NAME: &str = ACTOR_PUBKEY_PARTY_CONTRACT;
mod state;

actor_handlers! {
	codec::messaging::OP_DELIVER_MESSAGE => handle_message,
	tea_codec::ops::replica::OP_ACTOR_EXEC_TXN => handle_txn_exec,
	codec::core::OP_HEALTH_REQUEST => health
}

pub fn can_do() -> anyhow::Result<bool> {
	let miner_info = fetch_miner_info_remotely(get_my_tea_id()?)?;
	Ok(miner_info.class == MinerClass::A)
}

fn handle_message(msg: BrokerMessage) -> HandlerResult<Vec<u8>> {
	debug!("party state actor received deliver message, {:?}", &msg);

	match handle_message_inner(msg) {
		Ok(res) => Ok(res),
		Err(e) => {
			error!("party-state-actor handle test task error {}", e);
			Err(e)
		}
	}
}
fn handle_message_inner(msg: BrokerMessage) -> HandlerResult<Vec<u8>> {
	let channel_parts: Vec<&str> = msg.subject.split('.').collect();
	match &channel_parts[..] {
		["tea", "system", "init"] => handle_system_init()?,
		_ => (),
	};
	Ok(vec![])
}
fn handle_system_init() -> anyhow::Result<()> {
	info!("party-state-actor system init...");

	register_layer1_event()?;
	Ok(())
}

/// Get the state's last executed txn's tsid.
/// The next to-be-executed txn will be based on this tsid
/// This tsid is also used to determine if the state is out of date
/// If all state machine replica up to date, this tsid 
/// shoudl be consistent. 
fn helper_get_state_tsid() -> HandlerResult<Tsid> {
	let tsid_bytes: Vec<u8> = actor_statemachine::query_state_tsid()?;
	let tsid: Tsid = bincode::deserialize(&tsid_bytes)?;
	Ok(tsid)
}

/// this is the entrance of all txn (command) execution.
fn handle_txn_exec(msg: BrokerMessage) -> HandlerResult<()> {
	// 
	let (tsid, txn_bytes): (Tsid, Vec<u8>) = bincode::deserialize(&msg.body)?;
	// Call the txn_exec_inner in order to catch the Err.
	if let Err(e) = txn_exec_inner(tsid, &txn_bytes) {
		let txn_hash = tsid.hash;
		//report_txn_error can send back the error message to the caller (in hosting CML)
		report_txn_error(txn_hash.clone().to_vec(), e.to_string())?;
		return Err(format!(
			"party-state-actor handle txn exec err. hash: 0x{}) exec error: {}",
			hex::encode(txn_hash),
			e
		)
		.into());
	}
	Ok(())
}

/// calculate fee with msg ttl.
/// This is the billing model. 
/// If you want to change the billing mode, this is the place to change
fn calculate_fee(ttl: u32) -> Balance {
	let amt: Balance = if ttl > 40000 {
		3000000000000 as Balance
	} 
	else if ttl > 20000 {
		2000000000000 as Balance
	}
	else {
		1000000000000 as Balance
	};
	amt
}

/// This is the actualy command (txn) execution function entry
/// This function (inside the match clause) is where you business logic
/// located in the code
fn txn_exec_inner(tsid: Tsid, txn_bytes: &[u8]) -> HandlerResult<()> {
	// info!("before TeapartyTxn der");
	let sample_txn: TeapartyTxn = bincode::deserialize(txn_bytes)?;
	// info!("decode the txn {:?}", &sample_txn);
	let base: Tsid = helper_get_state_tsid()?;
	// info!("base tsid is {:?}", &base);
	let (context_bytes, auth_key): (Vec<u8>, AuthKey) = match sample_txn {
		/// PostMessage, when user post a new message
		TeapartyTxn::PostMessage {
			token_id,
			from,
			ttl,
			auth_b64,
		} => {
			info!("PostMessage => from ttl: {:?},{:?}", &from, &ttl);
			let amt = calculate_fee(ttl.try_into().unwrap());
			let auth_key: AuthKey = bincode::deserialize(&base64::decode(auth_b64)?)?;
			let auth_ops_bytes = actor_statemachine::query_auth_ops_bytes(auth_key)?;
			let ctx = TokenContext::new(tsid, base, token_id, &auth_ops_bytes)?;
			let req = ConsumeFromAccountRequest {
				ctx: bincode::serialize(&ctx)?,
				acct: bincode::serialize(&from)?,
				amt: bincode::serialize(&amt)?,
			};
			(actor_statemachine::consume_from_account(req)?, auth_key)
		}

		/// User click the extend message button to extend
		/// the life span of existing message
		/// of course he will need to pay
		TeapartyTxn::ExtendMessage {
			token_id,
			from,
			ttl,
			auth_b64,
		} => {
			info!("ExtendMessage => from ttl: {:?},{:?}", &from, &ttl);

			let amt = calculate_fee(ttl.try_into().unwrap());

			let auth_key: AuthKey = bincode::deserialize(&base64::decode(auth_b64)?)?;
			let auth_ops_bytes = actor_statemachine::query_auth_ops_bytes(auth_key)?;
			let ctx = TokenContext::new(tsid, base, token_id, &auth_ops_bytes)?;
			let req = ConsumeFromAccountRequest {
				ctx: bincode::serialize(&ctx)?,
				acct: bincode::serialize(&from)?,
				amt: bincode::serialize(&amt)?,
			};
			(actor_statemachine::consume_from_account(req)?, auth_key)
		}
		TeapartyTxn::DeleteMessage {
			token_id,
			from,
			auth_b64,
			is_tapp_owner,
		} => {
			info!("DeleteMessage => {:?}\n{:?}\n{:?}\n{:?}", token_id, from, auth_b64, is_tapp_owner);

			let amt = 0 as Balance;

			let auth_key: AuthKey = bincode::deserialize(&base64::decode(auth_b64)?)?;
			let auth_ops_bytes = actor_statemachine::query_auth_ops_bytes(auth_key)?;
			let ctx = TokenContext::new(tsid, base, token_id, &auth_ops_bytes)?;
			let req = ConsumeFromAccountRequest {
				ctx: bincode::serialize(&ctx)?,
				acct: bincode::serialize(&from)?,
				amt: bincode::serialize(&amt)?,
			};
			(actor_statemachine::consume_from_account(req)?, auth_key)
		}

		/// user send a private message with notification feature
		TeapartyTxn::AddNotificationMessage {
			token_id,
			from,
			to,
			auth_b64,
			current,
			ttl,
		} => {
			info!(
				"AddNotificationMessage => : {:?}\n{:?}\n{:?}\n{:?}\n{:?}",
				token_id, from, to, auth_b64, ttl
			);
			let pushnotifications_inner_request = tappstore::PushNotificationsInnerRequest {
				token_id,
				accounts: vec![public_key_to_ss58(&to)?],
				expired_heights: vec![ttl+current],
				uuid: "ok".to_string(),
				tsid: bincode::serialize(&tsid)?,
			};
			
			let amt = calculate_fee(ttl);

			let auth_key: AuthKey = bincode::deserialize(&base64::decode(auth_b64)?)?;
			let auth_ops_bytes = actor_statemachine::query_auth_ops_bytes(auth_key)?;
			let ctx = TokenContext::new(tsid, base, token_id, &auth_ops_bytes)?;
			let req = ConsumeFromAccountRequest {
				ctx: bincode::serialize(&ctx)?,
				acct: bincode::serialize(&from)?,
				amt: bincode::serialize(&amt)?,
			};

			info!("req: {:?}", &pushnotifications_inner_request);
			post_intercom(
				tea_codec::ACTOR_PUBKEY_TAPPSTORE,
				&BrokerMessage {
					subject: "actor.tappstore.push_notifications".into(),
					reply_to: "".into(),
					body: encode_protobuf(pushnotifications_inner_request).unwrap(),
				},
			)?;
			
			(actor_statemachine::consume_from_account(req)?, auth_key)
		}

		_ => Err(anyhow::anyhow!("Unhandled txn OP type"))?,
	};
	if context_bytes.is_empty() {
		error!("######### party state actor txn handle returns empty ctx. Cannot commit ######");
		return Ok(());
	}
	let hidden_acct_balance_change_after_commit = actor_statemachine::commit(CommitRequest {
		ctx: context_bytes,
		auth_key: bincode::serialize(&auth_key)?,
	})?;
	if hidden_acct_balance_change_after_commit != (0, 0) {
		warn!("********* party state actor commit succesfully but the hidden account balance changed. make sure a follow up tx is trigger to keey the balance sheet balance. {:?}", &hidden_acct_balance_change_after_commit);
	} else {
		info!("*********  party state actor commit succesfully.");
	}
	Ok(())
}
fn health(_req: codec::core::HealthRequest) -> HandlerResult<()> {
	info!("health call from party-state actor");
	Ok(())
}


