use crate::utils;
use aurora_engine_precompiles::promise_result::{self, costs};
use aurora_engine_transactions::legacy::TransactionLegacy;
use aurora_engine_types::{
    types::{Address, EthGas, NearGas, PromiseResult, Wei},
    U256,
};

const NEAR_GAS_PER_EVM: u64 = 175_000_000;

#[test]
fn test_promise_results_precompile() {
    let mut signer = utils::Signer::random();
    let mut runner = utils::deploy_runner();

    let promise_results = vec![
        PromiseResult::Successful(hex::decode("deadbeef").unwrap()),
        PromiseResult::Failed,
    ];

    let transaction = TransactionLegacy {
        nonce: signer.use_nonce().into(),
        gas_price: U256::zero(),
        gas_limit: u64::MAX.into(),
        to: Some(promise_result::ADDRESS),
        value: Wei::zero(),
        data: Vec::new(),
    };

    runner.promise_results.clone_from(&promise_results);
    let result = runner
        .submit_transaction(&signer.secret_key, transaction)
        .unwrap();

    assert_eq!(
        utils::unwrap_success(result),
        borsh::to_vec(&promise_results).unwrap(),
    );
}

#[test]
fn test_promise_result_gas_cost() {
    let mut runner = utils::deploy_runner();
    let mut signer = utils::Signer::random();
    // Skip to later block height and re-init hashchain
    let account_id = runner.aurora_account_id.clone();
    utils::init_hashchain(
        &mut runner,
        &account_id,
        Some(aurora_engine::engine::ZERO_ADDRESS_FIX_HEIGHT + 1),
    );

    // Baseline transaction that does essentially nothing.
    let (_, baseline) = runner
        .submit_with_signer_profiled(&mut signer, |nonce| TransactionLegacy {
            nonce,
            gas_price: U256::zero(),
            gas_limit: u64::MAX.into(),
            to: Some(Address::from_array([0; 20])),
            value: Wei::zero(),
            data: Vec::new(),
        })
        .unwrap();

    let mut profile_for_promises = |promise_data: Vec<PromiseResult>| -> (u64, u64, u64) {
        let input_length: usize = promise_data.iter().map(PromiseResult::size).sum();
        runner.promise_results = promise_data;
        let (submit_result, profile) = runner
            .submit_with_signer_profiled(&mut signer, |nonce| TransactionLegacy {
                nonce,
                gas_price: U256::zero(),
                gas_limit: u64::MAX.into(),
                to: Some(promise_result::ADDRESS),
                value: Wei::zero(),
                data: Vec::new(),
            })
            .unwrap();
        assert!(submit_result.status.is_ok());
        // Subtract off baseline transaction to isolate just precompile things
        (
            u64::try_from(input_length).unwrap(),
            profile.all_gas() - baseline.all_gas(),
            submit_result.gas_used,
        )
    };

    let promise_results = vec![
        PromiseResult::Successful(hex::decode("deadbeef").unwrap()),
        PromiseResult::Failed,
        PromiseResult::Successful(vec![1u8; 100]),
    ];

    let (x1, y1, evm1) = profile_for_promises(Vec::new());
    let (x2, y2, evm2) = profile_for_promises(promise_results);

    let cost_per_byte = (y2 - y1) / (x2 - x1);
    let base_cost = NearGas::new(y1 - cost_per_byte * x1);

    let base_cost = EthGas::new(base_cost.as_u64() / NEAR_GAS_PER_EVM);
    let cost_per_byte = cost_per_byte / NEAR_GAS_PER_EVM;

    assert!(
        utils::within_x_percent(
            10,
            base_cost.as_u64(),
            costs::PROMISE_RESULT_BASE_COST.as_u64(),
        ),
        "Incorrect promise_result base cost. Expected: {} Actual: {}",
        base_cost,
        costs::PROMISE_RESULT_BASE_COST
    );

    assert!(
        utils::within_x_percent(5, cost_per_byte, costs::PROMISE_RESULT_BYTE_COST.as_u64()),
        "Incorrect promise_result per byte cost. Expected: {} Actual: {}",
        cost_per_byte,
        costs::PROMISE_RESULT_BYTE_COST
    );

    let total_gas1 = y1 + baseline.all_gas();
    let total_gas2 = y2 + baseline.all_gas();

    assert!(
        utils::within_x_percent(36, evm1, total_gas1 / NEAR_GAS_PER_EVM),
        "Incorrect EVM gas used. Expected: {} Actual: {}",
        evm1,
        total_gas1 / NEAR_GAS_PER_EVM
    );
    assert!(
        utils::within_x_percent(36, evm2, total_gas2 / NEAR_GAS_PER_EVM),
        "Incorrect EVM gas used. Expected: {} Actual: {}",
        evm2,
        total_gas2 / NEAR_GAS_PER_EVM
    );
}
