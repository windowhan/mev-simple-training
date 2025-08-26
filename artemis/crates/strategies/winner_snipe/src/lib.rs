use artemis_core::{
    types::{Actions, Events, Strategy},
};
use async_trait::async_trait;
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::LocalWallet,
    types::{Address, Bytes, TransactionRequest, U256},
};
use std::sync::Arc;

/// WinnerSnipe strategy for front-running setWinner() calls
pub struct WinnerSnipe {
    target_contract: Address,
    my_address: Address,
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl WinnerSnipe {
    pub fn new(target_contract: Address, my_address: Address, client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Self {
        Self {
            target_contract,
            my_address,
            client,
        }
    }
}

#[async_trait]
impl Strategy<Events, Actions> for WinnerSnipe {
    async fn sync_state(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn process_event(&mut self, event: Events) -> Vec<Actions> {
        match event {
            Events::Transaction(tx) => {
                println!("Processing transaction: {:?}", tx.hash);
                println!("Transaction to: {:?}", tx.to);
                println!("Target contract: {:?}", self.target_contract);
                
                // Check if this transaction calls setWinner() on our target contract
                if tx.to == Some(self.target_contract) {
                    println!("Transaction targets our contract!");
                    
                    // setWinner() function selector is 0xed05084e
                    let set_winner_selector = "ed05084e";
                    let input_hex = hex::encode(&tx.input);
                    println!("Input data: {}", input_hex);
                    
                    if input_hex.starts_with(set_winner_selector) {
                        println!("DETECTED setWinner() call! Preparing front-run...");
                        
                        // This is a setWinner() call, let's front-run it!
                        let gas_price = tx.gas_price.unwrap_or(U256::from(20_000_000_000u64));
                        let boosted_gas_price = gas_price * 120 / 100; // 20% more gas
                        
                        println!("Original gas price: {} wei", gas_price);
                        println!("Boosted gas price: {} wei", boosted_gas_price);
                        
                        let front_run_tx = TransactionRequest::new()
                            .to(self.target_contract)
                            .data(Bytes::from(hex::decode(set_winner_selector).unwrap()))
                            .gas_price(boosted_gas_price)
                            .gas(100_000)
                            .value(0);
                        
                        // 직접 트랜잭션 전송!
                        let client = self.client.clone();
                        tokio::spawn(async move {
                            match client.send_transaction(front_run_tx, None).await {
                                Ok(tx_hash) => println!("Front-run transaction sent! Hash: {:?}", tx_hash.tx_hash()),
                                Err(e) => println!("Failed to send front-run transaction: {:?}", e),
                            }
                        });
                        
                        println!("Front-run transaction created and submitted!");
                        return vec![];
                    } else {
                        println!("Not a setWinner() call, ignoring...");
                    }
                } else {
                    println!("Transaction not for our target contract, skipping...");
                }
            }
            _ => {
                println!("Non-transaction event received, ignoring...");
            }
        }
        
        vec![]
    }
}
