use num_bigint::{BigInt, BigUint};
use num_traits::cast::ToPrimitive;
use tea_actor_utility::actor_crypto::{generate, public_key_to_ss58, public_key_from_ss58};
use tea_actor_utility::actor_enclave::generate_uuid;
use tea_codec::{deserialize, serialize};
use vmh_codec::message::structs_proto::{layer1, rpc};
use hex;

const BALANCE_STORAGE_INDEX: u32 = 3;

const BBS_OPERATION_ACCOUNT_KEY_PREFIX: &str = "bbs_operation_account_key";
const BBS_OPERATION_ACCOUNT_PUB_KEY_PREFIX: &str = "bbs_operation_account_pub_key";
const BBS_BALANCE_STATE_PREFIX: &str = "bbs_balance_state_prefix";

use party_shared::{TeapartyTxn};

use interface::{
  Hash, Followup, Ts,
  Account, Balance,
};

use crate::state;
use crate::help;

// pub(crate) fn prepare_tapp_balances(uuid: &str, tapp_id: u64) -> anyhow::Result<()> {
// 	let (pub_key, pri_key) = generate("sr25519".into())?;
// 	raft_set_value(
// 		&bbs_operation_account_key(tapp_id),
// 		&pri_key,
// 		BALANCE_STORAGE_INDEX,
// 		uuid,
// 	)?;
// 	raft_set_value(
// 		&bbs_operation_account_pub_key(tapp_id),
// 		&pub_key,
// 		BALANCE_STORAGE_INDEX,
// 		uuid,
// 	)?;
// 	set_balance_state(uuid, tapp_id, BalanceState::default())?;

// 	Ok(())
// }

// pub(crate) fn clean_tapp_balances(uuid: &str, tapp_id: u64) -> anyhow::Result<()> {
	

// 	Ok(())
// }



// pub(crate) fn get_balance_state(uuid: &str, tapp_id: u64) -> anyhow::Result<BalanceState> {
// 	let balance_state = raft_get_value(
// 		&tapp_balance_state_key(tapp_id),
// 		BALANCE_STORAGE_INDEX,
// 		uuid,
// 	)?;

// 	Ok(deserialize(&balance_state)?)
// }

// pub(crate) fn set_balance_state(
// 	uuid: &str,
// 	tapp_id: u64,
// 	balance_state: BalanceState,
// ) -> anyhow::Result<()> {
// 	raft_set_value(
// 		&tapp_balance_state_key(tapp_id),
// 		&serialize(balance_state)?,
// 		BALANCE_STORAGE_INDEX,
// 		uuid,
// 	)
// }

pub(crate) fn on_top_up(event: layer1::TappTopupEvent) -> anyhow::Result<()> {
	info!("received topup event, details: {:?}", event);

	// let npc_account = event.to_account;
	// // TODO check npc account

	// let tapp_id = event.tapp_id;
	// let amt = BigUint::from_bytes_le(&event.amount);

	// // TODO, how to fix this unwrap
	// let amt = amt.to_u128().unwrap();

	// let acct = state::parse_to_acct(&event.from_account)?;

	// info!("1111 => {:?}", tapp_id);
	// info!("2222 => {:?}", amt);
	// info!("3333 => {:?}", acct);

	// state::topup(acct, amt)?;


	Ok(())
}

// pub(crate) fn on_tapp_hosted(event: layer1::TappHostedEvent) -> anyhow::Result<()> {
// 	if !event.become_active {
// 		return Ok(());
// 	}

// 	// todo get uuid from layer1 event
// 	let uuid = generate_uuid()?;
// 	prepare_tapp_balances(&uuid, event.tapp_id)
// }

// pub(crate) fn on_tapp_unhosted(event: layer1::TappUnhostedEvent) -> anyhow::Result<()> {
// 	if !event.become_pending {
// 		return Ok(());
// 	}

// 	// todo get uuid from layer1 event
// 	let uuid = generate_uuid()?;
// 	clean_tapp_balances(&uuid, event.tapp_id)
// }

// pub(crate) fn withdraw() {}

// pub(crate) fn balance_state_transaction(
// 	uuid: &str,
// 	tapp_id: u64,
// 	op: BalanceOperation,
// ) -> anyhow::Result<()> {
// 	let mut balance_state = get_balance_state(uuid, tapp_id)?;
// 	balance_state.execute(&op)?;
// 	save_balance_log(&op)?;
// 	set_balance_state(uuid, tapp_id, balance_state)?;
// 	Ok(())
// }


pub(crate) fn bbs_operation_account_pub_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_OPERATION_ACCOUNT_PUB_KEY_PREFIX, tapp_id)
}

pub(crate) fn bbs_operation_account_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_OPERATION_ACCOUNT_KEY_PREFIX, tapp_id)
}

pub(crate) fn tapp_balance_state_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_BALANCE_STATE_PREFIX, tapp_id)
}
