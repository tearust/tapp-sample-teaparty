use num_bigint::BigUint;
use num_traits::identities::Zero;
use num_traits::{CheckedAdd, CheckedSub};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum BalanceOperation {
	Mint(String, BigUint),
	Burn(String, BigUint),
	Transfer(String, String, BigUint),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceState {
	tapp_id: u64,
	balances: HashMap<String, BigUint>,
}

impl Default for BalanceState {
	fn default() -> Self {
		BalanceState {
			tapp_id: 0,
			balances: HashMap::new(),
		}
	}
}

impl BalanceState {
	pub fn execute(&mut self, operation: &BalanceOperation) -> anyhow::Result<()> {
		match operation {
			BalanceOperation::Mint(address, amount) => {
				self.increase_balance(address.as_str(), amount)
			}
			BalanceOperation::Burn(address, amount) => {
				self.decrease_balance(address.as_str(), amount)
			}
			BalanceOperation::Transfer(from, to, amount) => {
				self.decrease_balance(from.as_str(), amount)?;
				self.increase_balance(to.as_str(), amount)
			}
		}
	}

	pub fn batch_execute(&mut self, operations: Vec<BalanceOperation>) -> anyhow::Result<()> {
		for op in operations {
			self.execute(&op)?;
		}
		Ok(())
	}

	fn increase_balance(&mut self, address: &str, amount: &BigUint) -> anyhow::Result<()> {
		match self.balances.get_mut(address) {
			Some(old) => {
				*old = old
					.checked_add(amount)
					.ok_or(anyhow::anyhow!("add error"))?
			}
			None => {
				self.balances.insert(address.to_string(), amount.clone());
			}
		}
		Ok(())
	}

	fn decrease_balance(&mut self, address: &str, amount: &BigUint) -> anyhow::Result<()> {
		match self.balances.get_mut(address) {
			Some(old) => {
				*old = old
					.checked_sub(amount)
					.ok_or(anyhow::anyhow!("sub error"))?;
				if old.is_zero() {
					self.balances.remove(address);
				}
			}
			None => warn!("decrease balance address {} not exist", address),
		}
		Ok(())
	}
}
