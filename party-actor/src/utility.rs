#![allow(unused_imports)]

use interface::Account;
use std::convert::TryInto;
use tea_actor_utility::actor_crypto::public_key_from_ss58;

mod p2p_send;
pub use p2p_send::*;

pub fn parse_to_acct(ss58_address: &str) -> anyhow::Result<Account> {
	let acct = public_key_from_ss58(&ss58_address)?;
	if acct.len() != 32 {
		return Err(anyhow::anyhow!("{}", "Invalid ss58 account."));
	}
	let acct: Account = acct.try_into().unwrap();

	Ok(acct)
}

pub fn uuid_cb_key(uuid: &str, stype: &str) -> String {
	let rs = format!("{}_msg_{}", stype, uuid);
	rs.to_string()
}
// pub fn cb_key_to_uuid(key: &str, stype: &str) -> String {
// 	let ss = format!("{}_msg_", stype);
// 	let rs = str::replace(key, &ss, "");
// 	rs.to_string()
// }
