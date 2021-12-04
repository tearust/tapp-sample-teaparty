use num_bigint::{BigInt, BigUint};
use num_traits::cast::ToPrimitive;
use tea_actor_utility::actor_crypto::{generate, public_key_to_ss58, public_key_from_ss58};
use tea_actor_utility::actor_enclave::generate_uuid;
use tea_codec::{deserialize, serialize};
use vmh_codec::message::structs_proto::{layer1, rpc};
use hex;


use party_shared::{TeapartyTxn};

use interface::{
  Hash, Followup, Ts,
  Account, Balance,
};

use crate::state;

pub(crate) fn on_top_up(event: layer1::TappTopupEvent) -> anyhow::Result<()> {
	info!("received topup event, details: {:?}", event);

	let npc_account = event.to_account;
	// TODO check npc account

	let tapp_id = event.tapp_id;
	let amt = BigUint::from_bytes_le(&event.amount);

	// TODO, how to fix this unwrap
	let amt = amt.to_u128().unwrap();

	let acct = state::parse_to_acct(&event.from_account)?;
	let block_str = event.height.to_string();

	info!("1111 => {:?}", tapp_id);
	info!("2222 => {:?}", amt);
	info!("3333 => {:?}", acct);

	state::topup(acct, amt, block_str)?;

	Ok(())
}


