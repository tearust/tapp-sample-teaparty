use serde::{Serialize, Deserialize};
use bincode;
use bincode::Result as SerdeResult;
use thiserror::Error;
use interface::{
	Account, Balance, 
};
use interface::txn::{Txn, TxnError, TxnSerial};

pub const HANDLED_BY_ACTOR_NAME: &str = "TeapartyTxn";

#[derive(Debug, Error, PartialEq)]
pub enum TeapartyTxnError{
	#[error("TeapartyTxnExecution error:'{0}")]
	ErrorMessage(String),
	#[error("Unknown error")]
	Unknown,
	#[error("Parsing txn bytes failed. This txn bytes is not a valid SampleTxn. Error:'{0}")]
	ParseFailed(String),
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TeapartyTxn{
	Topup {
		acct: Account,
		amt: Balance,
		uuid: String,
	},
	Withdraw {
		acct: Account,
		amt: Balance,
		uuid: String,
	},

	TransferTea{
		from: Account, 
		to:Account, 
		amt:Balance,
		uuid: String,
	},

	PostMessage {
		from: Account,
		ttl: u64,
		uuid: String,
	},

	ExtendMessage {
		from: Account,
		ttl: u64,
		uuid: String,
	},
	

}

impl Txn for TeapartyTxn{
	fn into_bytes(&self)->Vec<u8>{
		let txn_serial = TxnSerial{
			actor_name : HANDLED_BY_ACTOR_NAME.to_string(),
			bytes : bincode::serialize(&self).unwrap(),
		};
		bincode::serialize(&txn_serial).unwrap()
	}
	fn from_bytes(bytes:Vec<u8>)->SerdeResult<Self>{

		bincode::deserialize::<Self>(&bytes)
	}
	fn get_handler_actor()->String{
		HANDLED_BY_ACTOR_NAME.into()
	}
	fn deserialize_to_txn_serial(bytes: &[u8])->Result<TxnSerial, TxnError>{
		let txn_serial: TxnSerial = 
			bincode::deserialize(bytes)
			.map_err(|e| TxnError::ParseFailed(e.to_string()))?;
		Ok(txn_serial)
	}
}


