#[macro_use]
extern crate log;
use serde::{Serialize, Deserialize};
use bincode::Result as SerdeResult;
use wascc_actor::prelude::codec::messaging::BrokerMessage;
use wascc_actor::prelude::*;
use wascc_actor::HandlerResult;
use party_shared::{{TeapartyTxn}};
use prost::Message;
use tea_actor_utility::actor_statemachine;
use interface::{TOKEN_ID_TEA, Balance, Tsid,};
use token_state::token_context::TokenContext;
use vmh_codec::message::structs_proto::tokenstate::*;
actor_handlers! {
	codec::messaging::OP_DELIVER_MESSAGE => handle_message,
	tea_codec::OP_ACTOR_EXEC_TXN => handle_txn_exec,
	codec::core::OP_HEALTH_REQUEST => health
}
fn handle_message(msg: BrokerMessage) -> HandlerResult<Vec<u8>> {
	debug!("simple-actor received deliver message, {:?}", &msg);

	match handle_message_inner(msg) {
		Ok(res) => Ok(res),
		Err(e) => {
			error!("simple-actor handle test task error {}", e);
			Err(e)
		}
	}
}
fn handle_message_inner(msg: BrokerMessage) -> HandlerResult<Vec<u8>> {
	let channel_parts: Vec<&str> = msg.subject.split('.').collect();
	match &channel_parts[..] {
		["tea", "system", "init"] => handle_system_init()?,
		_ => (),
	};
	Ok(vec![])
}
fn handle_system_init() -> anyhow::Result<()> {
	info!("simple actor system init...");
	Ok(())
}


fn helper_get_state_tsid()->HandlerResult<Tsid>{
	let tsid_bytes: Vec<u8> = actor_statemachine::query_state_tsid()?;
	let tsid: Tsid = bincode::deserialize(&tsid_bytes)?;
	Ok(tsid)
}
fn handle_txn_exec(msg: BrokerMessage) -> HandlerResult<()> {
	info!("enter handle_txn_exec");
	let (tsid, txn_bytes):(Tsid, Vec<u8>) = bincode::deserialize(&msg.body)?;
	info!("before TeapartyTxn der");
	let sample_txn: TeapartyTxn = bincode::deserialize(&txn_bytes)?;
	info!("decode the txn {:?}", &sample_txn);
	let base: Tsid = helper_get_state_tsid()?;
	info!("base tsid is {:?}", &base);
	let context_bytes = match sample_txn {
		TeapartyTxn::Topup{acct, amt} =>{
			let ctx = TokenContext::new(tsid, base, TOKEN_ID_TEA);
			let ctx_bytes = bincode::serialize(&ctx)?;
			let to: u32 = acct;
			let amt: Vec<u8> = bincode::serialize(&amt)?;
			actor_statemachine::topup(TopupRequest{
				ctx: ctx_bytes,
				to,
				amt,
			})?
		},
		TeapartyTxn::PostMessage{from, ttl} => {
			info!("PostMessage from ttl: {:?},{:?}", &from, &ttl);
			
			// ttl > 2000, 2 TEA, else, 1 TEA
			let amt: Vec<u8> = {
				let cost: Balance = match ttl > 2000 {
					true => 2 as Balance,
					false => 1 as Balance,
				};

				bincode::serialize(&cost)?
			};

			// This account could be a npc to dividend to all of tapp stakers.
			// TODO if set acct to 0_u32, will return error 
			let to_acct = 999_u32;
			
			let ctx = TokenContext::new(tsid, base, TOKEN_ID_TEA);
			let ctx_bytes = bincode::serialize(&ctx)?;

			let mov = MoveRequest {
				ctx: ctx_bytes,
				from,
				to: to_acct,
				amt,
			};
			actor_statemachine::mov(mov)?
		},

		TeapartyTxn::TransferTea{from, to, amt} => {
			info!("TransferTea from to amt: {:?},{:?},{:?}", &from, &to, &amt);
			let ctx = TokenContext::new(tsid, base, TOKEN_ID_TEA);
			let ctx_bytes = bincode::serialize(&ctx)?;
			let amt: Vec<u8> = bincode::serialize(&amt)?;

			let mov = MoveRequest {
				ctx: ctx_bytes,
				from,
				to,
				amt,
			};
			actor_statemachine::mov(mov)?
		},
		
		_ =>Err(anyhow::anyhow!("Unhandled txn OP type"))?,
	};
	let res_commit_ctx_bytes = actor_statemachine::commit(CommitRequest{
		ctx: context_bytes
	})?;
	if res_commit_ctx_bytes.is_empty(){
		info!("*********  Commit succesfully. the ctx is empty. it is supposed to be empty");
	}
	Ok(())
}
fn health(_req: codec::core::HealthRequest) -> HandlerResult<()> {
	info!("health call from simple actor");
	Ok(())
}