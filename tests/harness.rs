use fuels::{prelude::*};
use fuels_abigen_macro::abigen;
use crate::Error::TransactionError;

// Load abi from json
abigen!(MyContract, "out/debug/maturity_minimal_example-abi.json");

#[tokio::test]
async fn calling_a_contract_function_respects_maturity() -> Result<(), Error> {
    let wallet = launch_provider_and_get_wallet().await;

    let provider = wallet.get_provider().unwrap().clone();

    assert_eq!(provider.latest_block_height().await?, 0);

    let id = Contract::deploy("./out/debug/maturity_minimal_example.bin", &wallet, TxParameters::default())
        .await
        .unwrap();

    let instance = MyContract::new(id.to_string(), wallet);

    let call_contract_using_maturity = |maturity| {
        let mut prepared_call = instance.test_function();
        prepared_call.tx_parameters.maturity = maturity;

        prepared_call.call()
    };


    assert_eq!(provider.latest_block_height().await?, 1);
    call_contract_using_maturity(2).await.err().expect("Should have failed since we're requiring a maturity greater than the current block height");

    assert_eq!(provider.latest_block_height().await?, 1);
    call_contract_using_maturity(1).await.expect("This should now pass since we're using a maturity <= current block height");

    Ok(())
}

#[tokio::test]
async fn deploying_a_contract_respects_maturity() -> anyhow::Result<()> {
    let wallet = launch_provider_and_get_wallet().await;

    let provider = wallet.get_provider().unwrap().clone();

    let deploy_the_contract_w_maturity = |maturity| {
        let params = TxParameters {
            maturity,
            ..TxParameters::default()
        };
        Contract::deploy("./out/debug/maturity_minimal_example.bin", &wallet, params)
    };

    // Only the genesis block present
    assert_eq!(provider.latest_block_height().await?, 0);
    let err = deploy_the_contract_w_maturity(1).await.err().expect("Should have failed because we're calling with maturity 1 at block height 0");
    assert!(is_transaction_maturity_error(&err));

    // The block height remains unchanged
    assert_eq!(provider.latest_block_height().await?, 0);

    // We're now adding a block by executing a noop script via `add_blocks`.
    add_blocks(&wallet, 1).await?;

    assert_eq!(provider.latest_block_height().await?, 1);
    // this fails with execution error: PredicateFailure
    deploy_the_contract_w_maturity(1).await.expect("This should have passed since we're now deploying with maturity == current block height");

    Ok(())
}

fn is_transaction_maturity_error(err: &Error) -> bool {
    matches!(err, TransactionError(msg) if msg.contains("TransactionMaturity"))
}
