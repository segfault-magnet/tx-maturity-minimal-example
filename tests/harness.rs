use fuels::{prelude::*};
use fuels::client::FuelClient;
use fuels_abigen_macro::abigen;

// Load abi from json
abigen!(MyContract, "out/debug/maturity_minimal_example-abi.json");

#[tokio::test]
async fn can_get_contract_id() {
    // Launch a local network and deploy the contract
    let wallet = launch_provider_and_get_single_wallet().await;

    let client = wallet.get_provider().unwrap().client.clone();

    assert_eq!(latest_block_height(&client).await, 0);

    let id = Contract::deploy("./out/debug/maturity_minimal_example.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    assert_eq!(latest_block_height(&client).await, 1);

    let instance = MyContract::new(id.to_string(), wallet);

    instance.test_function().call().await.unwrap();

    assert_eq!(latest_block_height(&client).await, 2);

    let mut call_w_maturity = instance.test_function();
    call_w_maturity.tx_parameters.maturity = 1;

    // 'Contract call error: Response errors; unexpected block execution error
    // VmExecution { error: ValidationError(TransactionMaturity),
    call_w_maturity.call().await.unwrap();
}

async fn latest_block_height(client: &FuelClient) -> u64 {
    client.chain_info().await.unwrap().latest_block.height.0
}
