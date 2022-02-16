use bincode;
use interface::{
	txn::{Transferable, Txn, TxnSerial},
	Account, AuthKey, Balance, TokenId,
};
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use tea_actor_utility::actor_statemachine::new_txn_serial;
use thiserror::Error;

pub const HANDLED_BY_ACTOR_NAME: &str = "TeapartyTxn";

#[derive(Debug, Error, PartialEq)]
pub enum TeapartyTxnError {
	#[error("TeapartyTxnExecution error:'{0}")]
	ErrorMessage(String),
	#[error("Unknown error")]
	Unknown,
	#[error("Parsing txn bytes failed. This txn bytes is not a valid SampleTxn. Error:'{0}")]
	ParseFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TeapartyTxn {
	TransferTea {
		from: Account,
		to: Account,
		amt: Balance,
		uuid: String,
		auth: AuthKey,
	},

	PostMessage {
		token_id: TokenId,
		from: Account,
		ttl: u64,
		auth_b64: String,
	},

	ExtendMessage {
		token_id: TokenId,
		from: Account,
		ttl: u64,
		auth_b64: String,
	},
	DeleteMessage {
		token_id: TokenId,
		from: Account,
		auth_b64: String,
		is_tapp_owner: bool,
	},

	UpdateProfile {
		acct: Account,
		token_id: TokenId,
		auth_b64: String,

		post_message_fee: Balance,
	},

	AddNotificationMessage {
		token_id: TokenId,
		from: Account,
		to: Account,
		auth_b64: String,
		ttl: u32,
	},
}

impl Transferable for TeapartyTxn {
	fn get_handler_actor() -> String {
		HANDLED_BY_ACTOR_NAME.into()
	}
}

impl TryFrom<TxnSerial> for TeapartyTxn {
	type Error = bincode::Error;

	fn try_from(value: TxnSerial) -> Result<Self, Self::Error> {
		bincode::deserialize::<Self>(value.bytes())
	}
}

impl TryInto<TxnSerial> for TeapartyTxn {
	type Error = anyhow::Error;

	fn try_into(self) -> Result<TxnSerial, Self::Error> {
		Ok(new_txn_serial(
			HANDLED_BY_ACTOR_NAME.to_string(),
			bincode::serialize(&self).unwrap(),
		)?)
	}
}

impl Txn<'static> for TeapartyTxn {}

impl TeapartyTxn {
	pub fn to_serial_bytes(self) -> anyhow::Result<Vec<u8>> {
		let serial: TxnSerial = self.try_into()?;
		let rtn = bincode::serialize(&serial)?;
		Ok(rtn)
	}
}
