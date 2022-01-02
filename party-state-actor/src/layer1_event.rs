use anyhow::anyhow;
use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;
use vmh_codec::message::structs_proto::layer1;

use crate::state;

pub(crate) fn on_top_up(event: layer1::TappTopupEvent) -> anyhow::Result<()> {
	info!("received topup event, details: {:?}", event);

	if false == confirm_receive_acct_is_my_app_acct(&event.to_account) {
		return Err(anyhow!(
			"this top up is not sent to my app layer one receiving account"
		));
	}

	let tapp_id = event.tapp_id;
	if false == confirm_app_id_is_myself(tapp_id) {
		return Err(anyhow!("This top up event is not for me"));
	}
	let amt = BigUint::from_bytes_le(&event.amount);

	let amt = amt.to_u128().ok_or(anyhow!("convert to u128 error"))?;

	let acct = state::parse_to_acct(&event.from_account)?;
	let block_str = event.height.to_string();
	state::topup(acct, amt, block_str)?;
	Ok(())
}

fn confirm_app_id_is_myself(_app_id: u64) -> bool {
	warn!("confirm_app_id_is_myself");
	true
}

fn confirm_receive_acct_is_my_app_acct(_to_account: &str) -> bool {
	warn!("todo confirm_receive_acct_is_my_app_acct");
	true
}
