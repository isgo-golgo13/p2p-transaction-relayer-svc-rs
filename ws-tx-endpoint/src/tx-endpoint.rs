use serde::{Deserialize, Serialize};
use crate::Transaction;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxEndpoint {
    pub id: String,
    pub balance: f64,
    pub transaction_count: u64,
}

impl TxEndpoint {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            balance: 1000.0, // Starting balance
            transaction_count: 0,
        }
    }

    pub fn process_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
        if tx.from == self.id {
            if self.balance < tx.amount {
                return Err("Insufficient balance".to_string());
            }
            self.balance -= tx.amount;
        } else if tx.to == self.id {
            self.balance += tx.amount;
        }
        
        self.transaction_count += 1;
        Ok(())
    }

    pub fn create_transaction(&self, to: &str, amount: f64) -> Transaction {
        Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            from: self.id.clone(),
            to: to.to_string(),
            amount,
            timestamp: js_sys::Date::now() as u64,
            signature: format!("sig_{}", self.transaction_count),
            status: "pending".to_string(),
        }
    }
}
