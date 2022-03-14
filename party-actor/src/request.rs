use interface::AuthKey;
use prost::Message;
use serde_json::json;
use std::collections::{HashMap, VecDeque};
use str_utils::*;
use tea_actor_utility::actor_crypto::{aes_decrypt, aes_encrypt};
use tea_actor_utility::actor_env::current_timestamp;
use tea_actor_utility::actor_raft::{
	raft_delete_value, raft_get_value, raft_get_values, raft_set_value,
};
use tea_actor_utility::common::calculate_hash;
use tea_codec;
use tea_codec::{deserialize, serialize};

use party_shared::TeapartyTxn;
use vmh_codec::message::{
	encode_protobuf,
	structs_proto::{orbitdb, replica, tappstore, tokenstate},
};
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;

use crate::help;
use crate::utility::{p2p_send_query, p2p_send_txn, uuid_cb_key};

pub fn send_txn(
	action_name: &str,
	uuid: &str,
	req_bytes: Vec<u8>,
	txn_bytes: Vec<u8>,
	txn_target: &str,
) -> anyhow::Result<()> {
	let ori_uuid = str::replace(&uuid, "txn_", "");
	let action_key = uuid_cb_key(&ori_uuid, "action_name");
	let req_key = uuid_cb_key(&ori_uuid, "action_req");
	help::set_mem_cache(&action_key, bincode::serialize(&action_name)?)?;
	help::set_mem_cache(&req_key, req_bytes.clone())?;

	info!(
		"start to send txn request for {} with uuid [{}]",
		&action_name, &uuid
	);
	p2p_send_txn(txn_bytes, uuid.to_string(), txn_target.to_string())?;
	info!("finish to send txn request...");

	Ok(())
}

pub fn send_query(query_bytes: Vec<u8>, uuid: &str, to_actor_name: String) -> anyhow::Result<()> {
	p2p_send_query(query_bytes, &uuid, to_actor_name)
}

pub fn to_query_uuid(uuid: &str) -> String {
	let query_uuid = str::replace(&uuid, "txn_", "");
	let query_uuid = str::replace(&query_uuid, "hash_", "");

	query_uuid.to_string()
}
