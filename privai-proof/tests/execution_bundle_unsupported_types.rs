use privai_chain::{
    ExecutionMode, ModelTx, SettlementTx, SettlementPhase, ModelAction, StakeAction,
    StakeTx, Transaction, TransferNoteTx, TxCore, PRIVAI_V0, TX_TYPE_LITE_TRANSFER, TX_TYPE_MODEL,
    TX_TYPE_SETTLEMENT, TX_TYPE_STAKE, TX_TYPE_TRANSFER_NOTE,
};
use privai_chain::small_payments::SettlementBatchSummary;
use privai_chain::tx::{LiteTransferTx, LiteTxCore, MarketplaceBatchTx, TX_TYPE_MARKETPLACE_BATCH};
use privai_proof::{
    build_execution_bundle_from_transactions, public_inputs_hash_for_transaction, BatchBuildError,
};

// ------------------------------------------------------------------------------------------------
// Local Test Helpers
// ------------------------------------------------------------------------------------------------

fn build_dummy_tx_core(tx_type: u8) -> TxCore {
    TxCore {
        version: PRIVAI_V0,
        tx_type,
        inputs: vec![],
        input_nullifiers: vec![],
        outputs: vec![],
        fee: 10,
        statement_commit: [0x55; 32],
        auth: Vec::new(),
    }
}

fn sample_settlement_tx() -> Transaction {
    Transaction::Settlement(SettlementTx {
        core: build_dummy_tx_core(TX_TYPE_SETTLEMENT),
        settlement_id: [1; 32],
        marketplace_context: [2; 16],
        phase: SettlementPhase::Open,
        payload_commit: [3; 32],
    })
}

fn sample_model_tx() -> Transaction {
    Transaction::Model(ModelTx {
        core: build_dummy_tx_core(TX_TYPE_MODEL),
        operator_pk_hash: [1; 32],
        model_commit: [2; 32],
        metadata_commit: [3; 32],
        action: ModelAction::Register,
    })
}

fn sample_stake_tx() -> Transaction {
    Transaction::Stake(StakeTx {
        core: build_dummy_tx_core(TX_TYPE_STAKE),
        validator_pk_hash: [1; 32],
        action: StakeAction::Bond,
        amount_delta: 0,
    })
}

fn sample_marketplace_batch_tx() -> Transaction {
    let summary = SettlementBatchSummary {
        operator_commit: [0; 32],
        merchant_commit: [2; 32],
        grant_commit: [3; 32],
        settlement_window_start: 0,
        settlement_window_end: 1000,
        receipt_root: [4; 32],
        receipt_count: 1,
        nullifier_count: 1,
        total_gross_amount: 100,
        total_fee_amount: 10,
        total_refund_amount: 0,
    };
    Transaction::MarketplaceBatch(MarketplaceBatchTx {
        core: build_dummy_tx_core(TX_TYPE_MARKETPLACE_BATCH),
        summary,
        ticket_nullifiers: vec![privai_chain::Nullifier([0xcc; 32])],
        operator_sig: Vec::new(),
    })
}

fn sample_lite_transfer_tx() -> Transaction {
    Transaction::LiteTransfer(LiteTransferTx {
        core: LiteTxCore {
            version: PRIVAI_V0,
            tx_type: TX_TYPE_LITE_TRANSFER,
            inputs: vec![],
            input_nullifiers: vec![],
            outputs: vec![],
            fee: 10,
            statement_commit: [0x55; 32],
            auth: Vec::new(),
        },
    })
}

fn sample_transfer_tx() -> Transaction {
    Transaction::TransferNote(TransferNoteTx {
        core: build_dummy_tx_core(TX_TYPE_TRANSFER_NOTE),
    })
}

// ------------------------------------------------------------------------------------------------
// 1. `public_inputs_hash_rejects_settlement_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn public_inputs_hash_rejects_settlement_tx() {
    let tx = sample_settlement_tx();
    let res = public_inputs_hash_for_transaction(&tx);

    assert!(
        matches!(res, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_SETTLEMENT),
        "Expected UnsupportedTransactionType for SettlementTx, got {:?}", res
    );
}

// ------------------------------------------------------------------------------------------------
// 2. `public_inputs_hash_rejects_model_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn public_inputs_hash_rejects_model_tx() {
    let tx = sample_model_tx();
    let res = public_inputs_hash_for_transaction(&tx);

    assert!(
        matches!(res, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_MODEL),
        "Expected UnsupportedTransactionType for ModelTx, got {:?}", res
    );
}

// ------------------------------------------------------------------------------------------------
// 3. `public_inputs_hash_rejects_stake_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn public_inputs_hash_rejects_stake_tx() {
    let tx = sample_stake_tx();
    let res = public_inputs_hash_for_transaction(&tx);

    assert!(
        matches!(res, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_STAKE),
        "Expected UnsupportedTransactionType for StakeTx, got {:?}", res
    );
}

// ------------------------------------------------------------------------------------------------
// 4. `public_inputs_hash_rejects_marketplace_batch_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn public_inputs_hash_rejects_marketplace_batch_tx() {
    let tx = sample_marketplace_batch_tx();
    let res = public_inputs_hash_for_transaction(&tx);

    assert!(
        matches!(res, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_MARKETPLACE_BATCH),
        "Expected UnsupportedTransactionType for MarketplaceBatchTx, got {:?}", res
    );
}

// ------------------------------------------------------------------------------------------------
// 5. `public_inputs_hash_rejects_lite_transfer_tx`
// ------------------------------------------------------------------------------------------------
#[test]
fn public_inputs_hash_rejects_lite_transfer_tx() {
    let tx = sample_lite_transfer_tx();
    let res = public_inputs_hash_for_transaction(&tx);

    assert!(
        matches!(res, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_LITE_TRANSFER),
        "Expected UnsupportedTransactionType for LiteTransferTx, got {:?}", res
    );
}

// ------------------------------------------------------------------------------------------------
// 6. `marketplace_only_batch_produces_housekeeping_bundle`
// ------------------------------------------------------------------------------------------------
#[test]
fn marketplace_only_batch_produces_housekeeping_bundle() {
    let tx = sample_marketplace_batch_tx();
    let txs = vec![tx];

    let bundle = build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof)
        .expect("bundle");

    assert_eq!(
        bundle.execution_mode,
        ExecutionMode::Housekeeping,
        "MarketplaceBatchTx should force Housekeeping mode"
    );
    assert!(bundle.statement_commits.is_empty());
    assert!(bundle.covered_tx_indexes.is_empty());
}

// ------------------------------------------------------------------------------------------------
// 7. `lite_transfer_in_transaction_batch_currently_returns_unsupported_transaction_type`
// ------------------------------------------------------------------------------------------------
#[test]
fn lite_transfer_in_transaction_batch_currently_returns_unsupported_transaction_type() {
    let tx = sample_lite_transfer_tx();
    let txs = vec![tx];

    let result = build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof);

    // LiteTransferTx is currently partially caught: it passes the outer filter 
    // `matches!(tx, Transaction::TransferNote(_) | Transaction::LiteTransfer(_))` 
    // but crashes in `public_inputs_hash_for_transaction`. This is a known current non-conformity.
    assert!(
        matches!(result, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_LITE_TRANSFER),
        "Expected UnsupportedTransactionType for LiteTransferTx in batch, got {:?}", result
    );
}

// ------------------------------------------------------------------------------------------------
// 8. `mixed_batch_with_lite_transfer_surfaces_current_lite_gap`
// ------------------------------------------------------------------------------------------------
#[test]
fn mixed_batch_with_lite_transfer_surfaces_current_lite_gap() {
    let tx_transfer = sample_transfer_tx();
    let tx_lite = sample_lite_transfer_tx();
    let txs = vec![tx_transfer, tx_lite];

    let result = build_execution_bundle_from_transactions(&txs, ExecutionMode::FullBatchProof);

    // This surfaces the lite gap - even if a proper TransferNoteTx exists, 
    // the presence of LiteTransferTx triggers an UnsupportedTransactionType error 
    // down the public_inputs_hash derivation path.
    assert!(
        matches!(result, Err(BatchBuildError::UnsupportedTransactionType { tx_type }) if tx_type == TX_TYPE_LITE_TRANSFER),
        "Expected UnsupportedTransactionType indicating the current lite gap, got {:?}", result
    );
}
