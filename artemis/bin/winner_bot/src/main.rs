use artemis_core::{
    collectors::mempool_collector::MempoolCollector,
    executors::mempool_executor::MempoolExecutor,
    engine::Engine,
    types::{CollectorMap, Events, Actions, ExecutorMap},
};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::Address,
};
use std::sync::Arc;
use winner_snipe::WinnerSnipe;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Starting Winner Bot!");
    
    let ws_url = "ws://127.0.0.1:8545";
    let target: Address = "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".parse()?;
    let me_addr: Address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".parse()?;
    let me_pk = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    
    println!("Target contract: {}", target);
    println!("Bot address: {}", me_addr);
    
    // Set up providers
    let ws_provider = Provider::<Ws>::connect(ws_url).await?;
    let provider = Arc::new(ws_provider);
    
    // Create signer for executor
    let wallet: LocalWallet = me_pk.parse()?;
    let wallet = wallet.with_chain_id(31337u64);
    
    // Create HTTP provider and signer middleware for executor  
    let http_provider = Provider::<Http>::try_from("http://127.0.0.1:8545")?;
    let client = Arc::new(SignerMiddleware::new(http_provider, wallet));
    
    // Create components
    let mempool_collector = MempoolCollector::new(provider);
    let collector = CollectorMap::new(
        Box::new(mempool_collector),
        |tx| Events::Transaction(tx)
    );
    
    let strategy = WinnerSnipe::new(target, me_addr, client.clone());
    
    let mempool_executor = MempoolExecutor::new(client);
    let executor = ExecutorMap::new(
        Box::new(mempool_executor),
        |action| match action {
            Actions::SubmitTxToMempool(submit_tx) => Some(submit_tx),
            _ => None,
        }
    );
    
    // Create and run engine
    println!("Setting up engine...");
    let mut engine = Engine::new();
    engine.add_collector(Box::new(collector));
    engine.add_strategy(Box::new(strategy));
    engine.add_executor(Box::new(executor));
    
    println!("Starting engine...");
    let mut join_set = engine.run().await.map_err(|e| anyhow::anyhow!("{:?}", e))?;
    
    println!("Listening for transactions...");
    // Wait for all tasks
    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            eprintln!("Task error: {:?}", e);
        }
    }
    
    Ok(())
}