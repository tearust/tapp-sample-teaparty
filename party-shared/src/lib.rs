use serde::{Serialize, Deserialize};
use bincode;
use bincode::Result as SerdeResult;
use thiserror::Error;
use interface::{
	Account, Balance, TOKEN_ID_TEA, Tsid, Operate, 
};
use interface::txn::{Txn, TxnError, TxnSerial};
use token_state::{
	token_state::{TokenState},
	token_context::{TokenContext},
};

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
	},
	Withdraw {
		acct: Account,
		amt: Balance,
	},

	PostMessage {
		acct: Account,
	},

	ExtendMessage {
		acct: Account,
		extend_time: Option<u64>,
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

// fn topup(
// 	ctx: &mut TokenContext,
// 	to: Account, 
// 	amt:Balance
// ) -> Result<TokenContext, TxnError> {
// 	state.topup(&mut ctx, to, amt).map_err(|e| TxnError::CausedByOperateError(e))
// }
// fn withdraw(
// 	ctx: &mut TokenContext,
// 	from: Account, 
// 	amt:Balance
// ) -> Result<TokenContext, TxnError> {
// 	state.withdraw(&mut ctx, from, amt).map_err(|e| TxnError::CausedByOperateError(e))
// }
// fn transfer(
// 	ctx: &mut TokenContext,
// 	from: Account, 
// 	to: Account, 
// 	amt: Balance,
// ) -> Result<TokenContext, TxnError> {
// 	state.move(&mut ctx, from, to, amt).map_err(|e| TxnError::CausedByOperateError(e))	
// }

// impl TeapartyTxn{
// 	fn execute(
// 		&self, 
// 		state: &TokenState, 
// 		tsid: Tsid, 
// 		base: Tsid,
// 	)->Result<TokenContext, TxnError>{

// 		let mut ctx = TokenContext::new(tsid, base, TOKEN_ID_TEA);
// 		match self {
// 			TeapartyTxn::Topup{acct, amt} => {
// 				topup(&mut ctx, *acct, *amt)
// 			},
// 			TeapartyTxn::Withdraw{acct, amt} => {
// 				withdraw(&mut ctx, *acct, amt)
// 			},
// 			TeapartyTxn::PostMessage{acct} => {
// 				let amt = 1 as Balance;
// 				move()
// 			}

// 			SampleTxn::Topup{acct, amt}=>Self::topup(state, tsid, base, *acct, *amt),
// 			SampleTxn::TransferTea{from, to, amt}=>Self::transfer_tea(state, tsid, base, *from, *to, *amt),
// 			SampleTxn::TestComboTopupTransferWithdraw{topup, to, amt}=>
// 				Self::test_combo_topup_transfer_withdraw(state, tsid, base, *topup, *to, *amt),
// 			_=> Err(TxnError::NewTxnTypeNotHandled)
// 		}
// 	}

// }


