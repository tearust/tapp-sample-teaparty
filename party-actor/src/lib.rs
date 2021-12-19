#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_camel_case_types)]

use crate::channel::{delete_message, extend_message, load_message_list, post_message};
use crate::validating::{login, logout, prepare_login};
use actor::prelude::*;
use codec::messaging::BrokerMessage;
use prost::Message;
#[cfg(not(feature = "nitro"))]
use tea_actor_utility::action::get_uuid;
#[cfg(feature = "nitro")]
use tea_actor_utility::actor_enclave::generate_uuid;
use tea_actor_utility::actor_layer1::register_layer1_event;
use tea_actor_utility::{
	actor_env::{get_system_time, get_env_var},
	action::reply_intercom, actor_rpc::register_adapter_http_dispatcher, wascc_actor as actor,
};
use types::*;
use vmh_codec::error::DISCARD_MESSAGE_ERROR;
use vmh_codec::message::structs_proto::{layer1, orbitdb, rpc, libp2p, tokenstate};
use vmh_codec::rpc::adapter::AdapterDispatchType;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod balance;
mod channel;
mod types;
mod validating;
mod state;
mod help;
mod user;

const BINDING_NAME: &'static str = "tea_tapp_bbs";
const VERSION: &str = env!("CARGO_PKG_VERSION");


actor_handlers! {
	codec::messaging::OP_DELIVER_MESSAGE => handle_message,
	codec::core::OP_HEALTH_REQUEST => health
}

fn handle_message(msg: BrokerMessage) -> HandlerResult<Vec<u8>> {
	// info!("bbs actor received deliver message, {:?}", &msg);

	let channel_parts: Vec<&str> = msg.subject.split('.').collect();
	match &channel_parts[..] {
		["tea", "system", "init"] => handle_system_init(&msg)?,
		["actor", "tapp_bbs", "echo", ..] => echo(&msg)?,
		["adapter", section] => return handle_adapter_request(msg.body.as_slice(), section),
		// ["layer1", "event"] => return handle_layer1_event(&msg.body),
		["actor", "version"] => version(&msg)?,
		["libp2p", "state-receiver", "back"] => return libp2p_back_message(&msg),
		_ => {}
	}
	Ok(vec![])
}

pub fn can_do() -> anyhow::Result<bool> {
	let miner_type = get_env_var("CML_TYPE")?;
	Ok(miner_type.eq("B"))
}

fn libp2p_back_message(msg: &BrokerMessage) -> HandlerResult<Vec<u8>> {
	let libp2p_request = libp2p::Libp2pRequest::decode(msg.body.as_slice())?;
	if let Some(libp2p::libp2p_request::Msg::GeneralRequest(r)) = libp2p_request.msg {

		let content = r.runtime_message
			.ok_or(anyhow::anyhow!("failed to get runtime message"))?
			.content;
		let body = 
			tokenstate::StateReceiverResponse::decode(content.as_slice())?;
		info!("party actor get lib msg back => {:?}", body);

		help::set_mem_cache(&body.uuid, content)?;
	}

	Ok(vec![])
}

fn health(_req: codec::core::HealthRequest) -> HandlerResult<()> {
	Ok(())
}

fn version(msg: &BrokerMessage) -> HandlerResult<()> {
	reply_intercom(
		&msg.reply_to,
		tea_codec::serialize(tea_codec::ActorVersionMessage {
			version: VERSION.to_string(),
		})?,
	)?;
	Ok(())
}

fn echo(msg: &BrokerMessage) -> HandlerResult<()> {
	debug!("echo received: {:?}", msg);
	if msg.reply_to.starts_with("reply") {
		let r = reply_intercom(&msg.reply_to, msg.body.clone());
		debug!("intercom response result {:?}", &r);
	}
	Ok(())
}

fn handle_system_init(_msg: &BrokerMessage) -> HandlerResult<()> {
	info!("tapp tea-party actor received tea.system.init");
	register_adapter_http_dispatcher(
		vec![
			"loginPrepare",
			"login",
			"logout",
			"postMessage",
			"loadMessageList",
			"extendMessage",
			"deleteMessage",
			"query_balance",
			"query_result",
			
			"test_action",
			"test_result",
		]
		.iter()
		.map(|v| v.to_string())
		.collect(),
	)?;
	// register_layer1_event()?;
	Ok(())
}

fn handle_layer1_event(data: &[u8]) -> HandlerResult<Vec<u8>> {
// 	info!("kkkkkkkkkkkk");
	if false == can_do()? {
		return Ok(vec![]);
	}

	let layer_inbound = layer1::Layer1Inbound::decode(data)?;

	let res = match layer_inbound.msg {
		Some(layer1::layer1_inbound::Msg::TappTopupEvent(ev)) => balance::on_top_up(ev),
		// Some(layer1::layer1_inbound::Msg::TappHostedEvent(ev)) => balance::on_tapp_hosted(ev),
		// Some(layer1::layer1_inbound::Msg::TappUnhostedEvent(ev)) => balance::on_tapp_unhosted(ev),
		_ => {
			debug!("ignored events: {:?}", layer_inbound.msg);
			Ok(())
		}
	};
	if let Err(e) = res {
		error!("process layer1 event error: {}", e);
	}

	Ok(vec![])
}

fn handle_adapter_request(data: &[u8], section: &str) -> HandlerResult<Vec<u8>> {
	let adapter_server_request = rpc::AdapterServerRequest::decode(data)?;
	debug!(
		"got adapter section: {}, request: {:?}",
		section, &adapter_server_request
	);
	match section {
		"http" => match adapter_server_request.msg {
			Some(rpc::adapter_server_request::Msg::AdapterHttpRequest(r)) => {
				info!("bbs got http request: {:?}", r);
				let res = handle_adapter_http_request(r)?;
				info!("bbs send response");
				return Ok(res);
			}
			_ => debug!(
				"ignored adapter ipfs server request message: {:?}",
				&adapter_server_request.msg
			),
		},
		_ => {
			debug!(
				"ignored adapter section ({}) message: {:?}",
				section, &adapter_server_request
			);
		}
	}
	Err(DISCARD_MESSAGE_ERROR.into())
}

fn handle_adapter_http_request(req: rpc::AdapterHttpRequest) -> anyhow::Result<Vec<u8>> {
	#[cfg(feature = "nitro")]
	let uuid = generate_uuid()?;
	#[cfg(not(feature = "nitro"))]
	let uuid = get_uuid();
	match req.action.as_str() {
		"loginPrepare" => {
			let req: PrepareLoginRequest = serde_json::from_slice(&req.payload)?;
			user::prepare_login_request(&req)
		}
		// "login" => {
		// 	let req: LoginRequest = serde_json::from_slice(&req.payload)?;
		// 	login(&uuid, &req)
		// }
		"logout" => {
			let req: LogoutRequest = serde_json::from_slice(&req.payload)?;
			logout(&uuid, &req)
		}
		"postMessage" => {
			let req: PostMessageRequest = serde_json::from_slice(&req.payload)?;
			post_message(&req.uuid.clone(), req.auth, req)
		}
		"loadMessageList" => {
			let req: LoadMessageRequest = serde_json::from_slice(&req.payload)?;
			load_message_list(&uuid, req)
		}
		"extendMessage" => {
			let req: ExtendMessageRequest = serde_json::from_slice(&req.payload)?;
			extend_message(&uuid, req)
		}
		"deleteMessage" => {
			let req: DeleteMessageRequest = serde_json::from_slice(&req.payload)?;
			delete_message(&uuid, req)
		}
		"query_balance" => {
			let req: HttpQueryBalanceRequest = serde_json::from_slice(&req.payload)?;

			state::query_tea_balance(&req.address, &req.uuid, req.auth)
		},
		"query_result" => {
			let req: HttpQueryResultWithUuid = serde_json::from_slice(&req.payload)?;

			let res_val = help::to_json_response(&req.uuid)?;
			Ok(serde_json::to_vec(&res_val)?)
		},
		"test_action" => {
			let req_json: serde_json::Value = serde_json::from_slice(&req.payload)?;
			info!("test action req => {:?}", req_json);

			let uuid = req_json["uuid"].as_str().unwrap();
			help::set_mem_cache(&uuid, req.payload)?;

			Ok(b"ok".to_vec())
		},
		"test_result" => {
			let req_json: serde_json::Value = serde_json::from_slice(&req.payload)?;
			info!("test result req => {:?}", req_json);

			let uuid = req_json["uuid"].as_str().unwrap();
			let rs = help::get_mem_cache(&uuid)?;

			Ok(rs)
		},
		_ => {
			debug!("unknown action: {}", req.action);
			Err(anyhow::anyhow!("{}", DISCARD_MESSAGE_ERROR))
		}
	}
}
