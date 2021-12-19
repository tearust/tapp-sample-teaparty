use crate::types::*;
use crate::BINDING_NAME;
use bincode;
use tea_actor_utility::{
	actor_crypto::{
		aes_decrypt, aes_encrypt, generate_aes_key, generate_rsa_keypair, public_key_from_ss58,
		public_key_to_ss58, rsa_decrypt, sign, verify,
	},
	actor_enclave::{get_my_ephemeral_id, get_my_ephemeral_key},
	actor_kvp,
};
use vmh_codec::{
	message::{
		structs_proto::{libp2p, tappstore, tokenstate},
		encode_protobuf,
	},
};
use serde_json::{json};

use crate::help;


pub fn save_session_key(
  session_key: String,
  tapp_id: &u64,
  address: &str,
) -> anyhow::Result<()> {
  let key = format!("session_key_{}_{}", tapp_id, address);

  actor_kvp::set(
		BINDING_NAME,
		&key,
		&session_key,
		6000*120,
	)
  .map_err(|e| anyhow::anyhow!("{}", e))?;
  
  Ok(())
}
pub fn get_session_key(
  tapp_id: &u64,
  address: &str,
) -> anyhow::Result<String> {
  let key = format!("session_key_{}_{}", tapp_id, address);

  let session_key: String = actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get session key"))?;

  Ok(session_key)
}

pub fn save_aes_key(
  aes_key: Vec<u8>,
  tapp_id: &u64,
) -> anyhow::Result<()> {
  let key = format!("aes_key_{}", tapp_id);
  actor_kvp::set_forever(
		BINDING_NAME,
		&key,
		&aes_key,
	)
  .map_err(|e| anyhow::anyhow!("{}", e))?;
  
  Ok(())
}
pub fn get_aes_key(
  tapp_id: &u64,
) -> anyhow::Result<Vec<u8>> {
  let key = format!("aes_key_{}", tapp_id);
  let aes_key: Vec<u8> = actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get aes key"))?;

  Ok(aes_key)
}


pub fn prepare_login_request(req: &PrepareLoginRequest) -> anyhow::Result<Vec<u8>> {
  // send to state receive actor

  let login_request = tappstore::TappQueryRequest {
    msg: Some(tappstore::tapp_query_request::Msg::LoginRequest(
      tappstore::LoginRequest {
        tapp_id: req.tapp_id,
        address: req.address.to_string(),
        data: base64::decode(&req.data)?,
        signature: base64::decode(&req.signature)?,
      }
    )),
  };

  help::p2p_send_to_receive_actor(encode_protobuf(
    tokenstate::StateReceiverMessage {
      uuid: req.uuid.to_string(),
      msg: Some(tokenstate::state_receiver_message::Msg::TappStoreCommand(
        tokenstate::TappStoreCommand {
          data: encode_protobuf(login_request)?
        }
      ))
    }
  )?)?;
  
  // mock
  // mock_login(&uuid, &req)?;
  Ok(b"ok".to_vec())
}

fn mock_login(uuid: &str, req: &PrepareLoginRequest) -> anyhow::Result<()> {

  // TODO aes_key and session_key come from tappstore actor in A.
  let aes_key: Vec<u8> = vec![8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];
  let session_key: String = "test_session_key".into();

  save_aes_key(aes_key, &req.tapp_id)?;
  save_session_key(session_key.clone(), &req.tapp_id, &req.address)?;

  let rs_json = json!({
    "session_key": session_key,
    "is_login": true,
  });
  help::set_mem_cache(&uuid, serde_json::to_vec(&rs_json)?)?;

  Ok(())
}