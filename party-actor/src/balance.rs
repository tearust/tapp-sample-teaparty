use crate::balance::operation::{BalanceOperation, BalanceState};
use num_bigint::{BigInt, BigUint};
use tea_actor_utility::actor_crypto::{generate, public_key_to_ss58};
use tea_actor_utility::actor_enclave::generate_uuid;
use tea_actor_utility::actor_raft::{raft_delete_value, raft_get_value, raft_set_value};
use tea_codec::{deserialize, serialize};
use vmh_codec::message::structs_proto::{layer1, rpc};

const BALANCE_STORAGE_INDEX: u32 = 3;

const BBS_OPERATION_ACCOUNT_KEY_PREFIX: &str = "bbs_operation_account_key";
const BBS_OPERATION_ACCOUNT_PUB_KEY_PREFIX: &str = "bbs_operation_account_pub_key";
const BBS_BALANCE_STATE_PREFIX: &str = "bbs_balance_state_prefix";

mod operation;

pub(crate) fn prepare_tapp_balances(uuid: &str, tapp_id: u64) -> anyhow::Result<()> {
	let (pub_key, pri_key) = generate("sr25519".into())?;
	raft_set_value(
		&bbs_operation_account_key(tapp_id),
		&pri_key,
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;
	raft_set_value(
		&bbs_operation_account_pub_key(tapp_id),
		&pub_key,
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;
	set_balance_state(uuid, tapp_id, BalanceState::default())?;

	Ok(())
}

pub(crate) fn clean_tapp_balances(uuid: &str, tapp_id: u64) -> anyhow::Result<()> {
	raft_delete_value(
		&bbs_operation_account_key(tapp_id),
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;
	raft_delete_value(
		&bbs_operation_account_pub_key(tapp_id),
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;
	raft_delete_value(
		&tapp_balance_state_key(tapp_id),
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;

	Ok(())
}

pub(crate) fn get_operation_account_address(uuid: &str, tapp_id: u64) -> anyhow::Result<String> {
	let pub_key = raft_get_value(
		&bbs_operation_account_pub_key(tapp_id),
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;

	Ok(public_key_to_ss58(pub_key.as_slice())?)
}

pub(crate) fn get_balance_state(uuid: &str, tapp_id: u64) -> anyhow::Result<BalanceState> {
	let balance_state = raft_get_value(
		&tapp_balance_state_key(tapp_id),
		BALANCE_STORAGE_INDEX,
		uuid,
	)?;

	Ok(deserialize(&balance_state)?)
}

pub(crate) fn set_balance_state(
	uuid: &str,
	tapp_id: u64,
	balance_state: BalanceState,
) -> anyhow::Result<()> {
	raft_set_value(
		&tapp_balance_state_key(tapp_id),
		&serialize(balance_state)?,
		BALANCE_STORAGE_INDEX,
		uuid,
	)
}

pub(crate) fn on_top_up(event: layer1::TappTopupEvent) -> anyhow::Result<()> {
	info!("received topup event, details: {:?}", event);

	let uuid = generate_uuid()?;
	match get_operation_account_address(&uuid, event.tapp_id) {
		Ok(address) => {
			if address.eq(&event.to_account) {
				let mint = BalanceOperation::Mint(address, BigUint::from_bytes_le(&event.amount));
				balance_state_transaction(uuid.as_str(), event.tapp_id, mint)?;
			}
		}
		Err(e) => {
			warn!("failed to find operation address of tapp(), error: {}", e);
		}
	}

	Ok(())
}

pub(crate) fn on_tapp_hosted(event: layer1::TappHostedEvent) -> anyhow::Result<()> {
	if !event.become_active {
		return Ok(());
	}

	// todo get uuid from layer1 event
	let uuid = generate_uuid()?;
	prepare_tapp_balances(&uuid, event.tapp_id)
}

pub(crate) fn on_tapp_unhosted(event: layer1::TappUnhostedEvent) -> anyhow::Result<()> {
	if !event.become_pending {
		return Ok(());
	}

	// todo get uuid from layer1 event
	let uuid = generate_uuid()?;
	clean_tapp_balances(&uuid, event.tapp_id)
}

pub(crate) fn withdraw() {}

pub(crate) fn balance_state_transaction(
	uuid: &str,
	tapp_id: u64,
	op: BalanceOperation,
) -> anyhow::Result<()> {
	let mut balance_state = get_balance_state(uuid, tapp_id)?;
	balance_state.execute(&op)?;
	save_balance_log(&op)?;
	set_balance_state(uuid, tapp_id, balance_state)?;
	Ok(())
}

pub(crate) fn save_balance_log(op: &BalanceOperation) -> anyhow::Result<()> {
	// todo save balance operation in client through vmh provider
	Ok(())
}

pub(crate) fn bbs_operation_account_pub_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_OPERATION_ACCOUNT_PUB_KEY_PREFIX, tapp_id)
}

pub(crate) fn bbs_operation_account_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_OPERATION_ACCOUNT_KEY_PREFIX, tapp_id)
}

pub(crate) fn tapp_balance_state_key(tapp_id: u64) -> String {
	format!("{}-{}", BBS_BALANCE_STATE_PREFIX, tapp_id)
}
