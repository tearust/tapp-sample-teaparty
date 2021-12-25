use crate::types::*;
use crate::BINDING_NAME;
use actor_txns::tappstore::TappstoreTxn;
use bincode;
use interface::{AuthKey, Followup, Ts};
use prost::Message;
use serde_json::json;
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
    structs_proto::{libp2p, tappstore, tokenstate},
};

use crate::help;
use crate::state;

pub fn save_session_key(session_key: Vec<u8>, tapp_id: &u64, address: &str) -> anyhow::Result<()> {
    let key = format!("session_key_{}_{}", tapp_id, address);

    actor_kvp::set(BINDING_NAME, &key, &session_key, 6000 * 120)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
pub fn get_session_key(tapp_id: &u64, address: &str) -> anyhow::Result<String> {
    let key = format!("session_key_{}_{}", tapp_id, address);

    let session_key: String =
        actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get session key"))?;

    Ok(session_key)
}

pub fn save_aes_key(aes_key: Vec<u8>, tapp_id: &u64) -> anyhow::Result<()> {
    let key = format!("aes_key_{}", tapp_id);
    actor_kvp::set_forever(BINDING_NAME, &key, &aes_key).map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
pub fn get_aes_key(tapp_id: &u64) -> anyhow::Result<Vec<u8>> {
    let key = format!("aes_key_{}", tapp_id);
    let aes_key: Vec<u8> =
        actor_kvp::get(BINDING_NAME, &key)?.ok_or(anyhow::anyhow!("failed to get aes key"))?;

    Ok(aes_key)
}

pub fn prepare_login_request(req: &PrepareLoginRequest) -> anyhow::Result<Vec<u8>> {
    // send to state receive actor
    let uuid = req.uuid.to_string();

    let query_bytes = tappstore::tapp_query_request::Msg::CheckUserSessionRequest(
        tappstore::CheckUserSessionRequest {
            account: req.address.to_string(),
            token_id: req.tapp_id,
            tea_id: help::get_tea_id()?,
        },
    );
    let query_bytes = tappstore::TappQueryRequest {
        msg: Some(query_bytes),
    };
    help::set_mem_cache(
        &help::uuid_cb_key(&uuid, &"check_user_auth"),
        encode_protobuf(query_bytes)?,
    )?;

    let login_request_txn = TappstoreTxn::GenSessionKey {
        token_id: req.tapp_id,
        acct_s58: req.address.to_string(),
        data: base64::decode(&req.data)?,
        signature: base64::decode(&req.signature)?,
        tea_id: help::get_tea_id()?,
    };
    let txn_bytes: Vec<u8> = bincode::serialize(&login_request_txn)?;
    let (sent_time, txn_hash) = state::send_tappstore_tx_via_p2p(
        txn_bytes,
        uuid.clone(),
        tea_codec::ACTOR_PUBKEY_TAPPSTORE.to_string(),
    )?;

    let sender_actor_hash = state::send_actor_hash()?;
    let req_fu: Followup = Followup {
        ts: sent_time,
        hash: txn_hash,
        sender: sender_actor_hash,
    };
    state::send_followup_via_p2p(req_fu, uuid.clone())?;

    // mock
    // mock_login(&uuid, &req)?;
    Ok(b"ok".to_vec())
}

pub fn libp2p_msg_cb(body: &tokenstate::StateReceiverResponse) -> anyhow::Result<bool> {
    let uuid = &body.uuid;

    // TODO check start string is check_user
    if uuid.len() > 46 {
        match &body.msg {
            Some(tokenstate::state_receiver_response::Msg::GeneralQueryResponse(r)) => {
                let query_res = tappstore::TappQueryResponse::decode(r.data.as_slice())?;

                libp2p_msg_cb_handler(&query_res)?;

                return Ok(true);
            }
            _ => warn!("unknown state receiver response: {:?}", body.msg),
        }
    }

    Ok(false)
}

fn libp2p_msg_cb_handler(res: &tappstore::TappQueryResponse) -> anyhow::Result<()> {
    match &res.msg {
        Some(tappstore::tapp_query_response::Msg::CheckUserSessionResponse(r)) => {
            let aes_key = &r.aes_key;
            let auth_key = &r.auth_key;

            // TODO save
            save_session_key(auth_key.to_vec(), &r.token_id, &r.account)?;
            save_aes_key(aes_key.to_vec(), &r.token_id)?;
        }
        _ => warn!("unknown tapp_query_response: {:?}", res.msg),
    }

    Ok(())
}

pub fn check_user_query_uuid(uuid: &str) -> String {
    format!("check_user_{}", uuid)
}