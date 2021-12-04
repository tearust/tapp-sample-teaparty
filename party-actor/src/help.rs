use prost::Message;

use tea_actor_utility::{
	actor_enclave::{generate_uuid, get_my_tea_id},
	actor_env::{get_system_time, time_since},
	actor_kvp,
  actor_libp2p,
  actor_layer1::general_remote_request,
};

use vmh_codec::message::{
  encode_protobuf, 
  structs_proto::{libp2p, layer1},
};
use wascc_actor::untyped;
use tea_codec;
use base64;

use crate::BINDING_NAME;


pub fn p2p_send_to_receive_actor(
  msg: Vec<u8>, 
)-> anyhow::Result<()> {
  // let my_tea_id = get_my_tea_id()?;
  let A_nodes = get_all_active_a_nodes()?;

  info!("all A nodes => {:?}", A_nodes);

  if(A_nodes.len() < 1){
    return Err(anyhow::anyhow!("{}", "No active A nodes."));
  }

  let target_conn_id = conn_id_by_tea_id(A_nodes[0].clone())?;
  info!("target conn id => {:?}", target_conn_id);
  
  // TOOD send to at least 2 A node.


  let target_key = tea_codec::ACTOR_PUBKEY_STATE_RECEIVER.to_string();
  let target_type = libp2p::TargetType::Actor as i32;

  let from_key = tea_codec::ACTOR_PUBKEY_TAPP_BBS.to_string();

  // TODO, convert to send

  info!("p2p send msg start...");
  actor_libp2p::send_message(
    target_conn_id,
    libp2p::RuntimeAddress {
      target_key,
      target_type,
      target_action: "libp2p.state-receiver".to_string(),
    },
    Some(libp2p::RuntimeAddress {
      target_key: from_key,
      target_type,
      target_action: Default::default(), // not needed
    }),
    msg,
  )?;
  info!("p2p send msg finish...");

  Ok(())
}


pub fn get_all_active_a_nodes() -> anyhow::Result<Vec<Vec<u8>>> {
	let res_buf = general_remote_request(layer1::Layer1Outbound {
		msg: Some(layer1::layer1_outbound::Msg::ListMiningCmlsRequest(
			layer1::ListMiningCmlsRequest {},
		)),
	})?;
	let res = layer1::ListMiningCmlsResponse::decode(res_buf.as_slice())?;

	let current_a_miners: Vec<Vec<u8>> = res
		.mining_cmls
		.iter()
		.filter(|info| info.cml_type.eq("A") && info.miner_status.eq("Active"))
		.map(|info| info.tea_id.clone())
		.collect();
	Ok(current_a_miners)
}

pub fn conn_id_by_tea_id(tea_id: Vec<u8>) -> anyhow::Result<String> {
	let res_buf = general_remote_request(layer1::Layer1Outbound {
		msg: Some(layer1::layer1_outbound::Msg::GetConnIdRequest(
			layer1::GetConnIdRequest { tea_id },
		)),
	})?;
  let res = layer1::GetConnIdResponse::decode(res_buf.as_slice())?;
	Ok(res.conn_id)
}

pub fn set_mem_cache(
  key: &str,
  val: Vec<u8>,
) -> anyhow::Result<()> {
  actor_kvp::set(
    BINDING_NAME,
    &key,
    &val,
    600,
  ).map_err(|e| anyhow::anyhow!("{}", e))?;

  Ok(())
}

pub fn get_mem_cache(key: &str) -> anyhow::Result<Vec<u8>> {
  let rs: Vec<u8> = actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get value with {}", key))?;

  Ok(rs)
}

