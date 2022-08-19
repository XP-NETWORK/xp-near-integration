use ed25519_dalek::Keypair;
use rand_core::OsRng;
use serde_json::json;
use workspaces::{
    network::{NetworkClient, NetworkInfo},
    Contract, Worker,
};

async fn get_group_key<T: NetworkClient + NetworkInfo + Send + Sync>(
    contract: &Contract,
    worker: &Worker<T>,
) -> [u8; 32] {
    contract
        .call(worker, "get_group_key")
        .args_json(json!({}))
        .unwrap()
        .view()
        .await
        .unwrap()
        .json()
        .unwrap()
}

#[tokio::test]
async fn test_xp_bridge() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;

    let wasm = include_bytes!("../res/bridge.wasm");

    let root_account = worker.root_account()?;
    println!("Root account: {:?}", root_account.id());

    let bridge_account = root_account
        .create_subaccount(&worker, "bridge")
        .initial_balance(10_000_000_000_000_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .unwrap();
    println!("Bridge account: {:?}", bridge_account.id());

    let bridge_contract = bridge_account.deploy(&worker, wasm).await?.unwrap();

    let mut csprng = OsRng {};
    let kp = Keypair::generate(&mut csprng);
    let group_key: [u8; 32] = kp.public.to_bytes();

    bridge_account
        .call(&worker, bridge_contract.id(), "initialize")
        .args_json(json!({ "group_key": group_key }))?
        .transact()
        .await
        .unwrap();

    assert_eq!(get_group_key(&bridge_contract, &worker).await, group_key);
    Ok(())
}
