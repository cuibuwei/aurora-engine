use crate::account::Account;
use crate::node::Node;
use crate::operation::{
    CallAddEntryToWhitelist, CallAddEntryToWhitelistBatch, CallAddRelayerKey,
    CallAttachFullAccessKey, CallCall, CallDeployCode, CallDeployErc20Token, CallDeployUpgrade,
    CallDeposit, CallFactorySetWNearAddress, CallFactoryUpdate, CallFactoryUpdateAddressVersion,
    CallFtOnTransfer, CallFtTransfer, CallFtTransferCall, CallFundXccSubAccount, CallMintAccount,
    CallMirrorErc20Token, CallNew, CallNewEthConnector, CallPauseContract, CallPausePrecompiles,
    CallRefundOnError, CallRegisterRelayer, CallRemoveEntryFromWhitelist, CallRemoveRelayerKey,
    CallResumeContract, CallResumePrecompiles, CallSetErc20Metadata,
    CallSetEthConnectorContractAccount, CallSetEthConnectorContractData, CallSetFixedGas,
    CallSetKeyManager, CallSetOwner, CallSetPausedFlags, CallSetSiloParams, CallSetWhitelistStatus,
    CallStageUpgrade, CallStateMigration, CallStorageDeposit, CallStorageUnregister,
    CallStorageWithdraw, CallSubmit, CallUpgrade, CallWithdraw, ViewAccountsCounter, ViewBalance,
    ViewBlockHash, ViewBridgeProver, ViewChainId, ViewCode, ViewErc20FromNep141,
    ViewFactoryWnearAddress, ViewFtBalanceOf, ViewFtBalanceOfEth, ViewFtBalancesOf, ViewFtMetadata,
    ViewFtTotalEthSupplyOnAurora, ViewFtTotalEthSupplyOnNear, ViewFtTotalSupply,
    ViewGetErc20Metadata, ViewGetEthConnectorContractAccount, ViewGetFixedGas, ViewGetSiloParams,
    ViewGetWhitelistStatus, ViewIsUsedProof, ViewNep141FromErc20, ViewNonce, ViewOwner,
    ViewPausedFlags, ViewPausedPrecompiles, ViewStorageAt, ViewStorageBalanceOf, ViewUpgradeIndex,
    ViewVersion, ViewView,
};
use crate::transaction::{CallTransaction, ViewTransaction};
use aurora_engine_types::account_id::AccountId;
use aurora_engine_types::parameters::connector::{
    Erc20Identifier, FungibleTokenMetadata, MirrorErc20TokenArgs, PausedMask, Proof,
    SetErc20MetadataArgs, SetEthConnectorContractAccountArgs, WithdrawSerializeType,
};
use aurora_engine_types::parameters::engine::{
    CallArgs, FullAccessKeyArgs, FunctionCallArgsV2, NewCallArgs, NewCallArgsV2, RelayerKeyArgs,
    RelayerKeyManagerArgs,
};
use aurora_engine_types::parameters::silo::{
    FixedGasArgs, SiloParamsArgs, WhitelistArgs, WhitelistKindArgs, WhitelistStatusArgs,
};
use aurora_engine_types::parameters::xcc::FundXccArgs;
use aurora_engine_types::public_key::PublicKey;
use aurora_engine_types::types::{Address, RawU256, WeiU256};
use aurora_engine_types::{H256, U256};
use near_sdk::json_types::U128;
use near_workspaces::types::SecretKey;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct EngineContract {
    contract: RawContract,
    public_key: PublicKey,
    pub node: Node,
}

impl EngineContract {
    pub fn as_raw_contract(&self) -> &RawContract {
        &self.contract
    }

    pub fn id(&self) -> AccountId {
        self.contract.id()
    }

    pub fn root(&self) -> Account {
        self.node.root()
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn create_account(&self, account_id: &AccountId, secret_key: SecretKey) -> Account {
        let inner = near_workspaces::Account::from_secret_key(
            account_id.as_ref().parse().unwrap(),
            secret_key,
            self.node.worker(),
        );

        Account::from_inner(inner)
    }
}

impl From<(RawContract, PublicKey, Node)> for EngineContract {
    fn from((contract, public_key, node): (RawContract, PublicKey, Node)) -> Self {
        Self {
            contract,
            public_key,
            node,
        }
    }
}

/// Callable functions implementation.
impl EngineContract {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        &self,
        chain_id: RawU256,
        owner_id: AccountId,
        upgrade_delay_blocks: u64,
    ) -> CallNew {
        let args = NewCallArgs::V2(NewCallArgsV2 {
            chain_id,
            owner_id,
            upgrade_delay_blocks,
        });

        CallNew::call(&self.contract).args_borsh(args)
    }

    pub fn new_eth_connector(
        &self,
        prover_account: AccountId,
        custodian_address: String,
        metadata: FungibleTokenMetadata,
    ) -> CallNewEthConnector {
        CallNewEthConnector::call(&self.contract).args_borsh((
            prover_account,
            custodian_address,
            metadata,
        ))
    }

    pub fn ft_transfer(
        &self,
        receiver_id: &AccountId,
        amount: U128,
        memo: Option<String>,
    ) -> CallFtTransfer {
        CallFtTransfer::call(&self.contract)
            .args_json(json!({ "receiver_id": receiver_id, "amount": amount, "memo": memo }))
    }

    pub fn ft_transfer_call(
        &self,
        receiver_id: &AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> CallFtTransferCall {
        CallFtTransferCall::call(&self.contract).args_json(json!({
           "receiver_id": receiver_id,
           "amount": amount,
           "memo": memo,
           "msg": msg,
        }))
    }

    pub fn storage_deposit(
        &self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> CallStorageDeposit {
        CallStorageDeposit::call(&self.contract)
            .args_json(json!({ "account_id": account_id, "registration_only": registration_only}))
    }

    pub fn storage_withdraw(&self, amount: Option<U128>) -> CallStorageWithdraw {
        CallStorageWithdraw::call(&self.contract).args_json(json!({ "amount": amount }))
    }

    pub fn storage_unregister(&self, force: Option<bool>) -> CallStorageUnregister {
        CallStorageUnregister::call(&self.contract).args_json(json!({ "force": force }))
    }

    pub fn withdraw(&self, recipient_address: Address, amount: u128) -> CallWithdraw {
        CallWithdraw::call(&self.contract).args_borsh((recipient_address, amount))
    }

    pub fn deposit(&self, raw_proof: Proof) -> CallDeposit {
        CallDeposit::call(&self.contract).args_borsh(raw_proof)
    }

    pub fn set_eth_connector_contract_data(
        &self,
        prover_account: AccountId,
        eth_custodian_address: String,
        metadata: FungibleTokenMetadata,
    ) -> CallSetEthConnectorContractData {
        CallSetEthConnectorContractData::call(&self.contract).args_borsh((
            prover_account,
            eth_custodian_address,
            metadata,
        ))
    }

    pub fn set_eth_connector_contract_account(
        &self,
        account_id: AccountId,
        withdraw_serialize_type: WithdrawSerializeType,
    ) -> CallSetEthConnectorContractAccount {
        CallSetEthConnectorContractAccount::call(&self.contract).args_borsh(
            SetEthConnectorContractAccountArgs {
                account: account_id,
                withdraw_serialize_type,
            },
        )
    }

    pub fn factory_update_address_version(
        &self,
        address: Address,
        version: u32,
    ) -> CallFactoryUpdateAddressVersion {
        CallFactoryUpdateAddressVersion::call(&self.contract).args_borsh((address, version))
    }

    pub fn refund_on_error(
        &self,
        recipient_address: Address,
        erc20_address: Option<Address>,
        amount: U256,
    ) -> CallRefundOnError {
        CallRefundOnError::call(&self.contract).args_borsh((
            recipient_address,
            erc20_address,
            amount.0,
        ))
    }

    pub fn deploy_code(&self, code: Vec<u8>) -> CallDeployCode {
        CallDeployCode::call(&self.contract).args(code)
    }

    pub fn deploy_erc20_token(&self, account_id: AccountId) -> CallDeployErc20Token {
        CallDeployErc20Token::call(&self.contract).args_borsh(account_id)
    }

    pub fn mirror_erc20_token(&self, args: MirrorErc20TokenArgs) -> CallMirrorErc20Token {
        CallMirrorErc20Token::call(&self.contract).args_borsh(args)
    }

    pub fn call(&self, contract: Address, amount: U256, input: Vec<u8>) -> CallCall {
        let value = WeiU256::from(amount);
        let args = CallArgs::V2(FunctionCallArgsV2 {
            contract,
            value,
            input,
        });
        CallCall::call(&self.contract).args_borsh(args)
    }

    pub fn submit(&self, input: Vec<u8>) -> CallSubmit {
        CallSubmit::call(&self.contract).args(input)
    }

    pub fn register_relayer(&self, address: Address) -> CallRegisterRelayer {
        CallRegisterRelayer::call(&self.contract).args_borsh(address)
    }

    pub fn ft_on_transfer(
        &self,
        sender_id: AccountId,
        amount: U128,
        message: String,
    ) -> CallFtOnTransfer {
        CallFtOnTransfer::call(&self.contract).args_json(json!({
            "sender_id": sender_id,
            "amount": amount,
            "message": message
        }))
    }

    pub fn factory_update(&self, bytes: Vec<u8>) -> CallFactoryUpdate {
        CallFactoryUpdate::call(&self.contract).args(bytes)
    }

    pub fn fund_xcc_sub_account(
        &self,
        target: Address,
        wnear_account_id: Option<AccountId>,
    ) -> CallFundXccSubAccount {
        let args = FundXccArgs {
            target,
            wnear_account_id,
        };
        CallFundXccSubAccount::call(&self.contract).args_borsh(args)
    }

    pub fn factory_set_wnear_address(&self, address: Address) -> CallFactorySetWNearAddress {
        CallFactorySetWNearAddress::call(&self.contract).args_borsh(address)
    }

    pub fn upgrade(&self, bytes: Vec<u8>) -> CallUpgrade {
        CallUpgrade::call(&self.contract).args(bytes)
    }

    pub fn stage_upgrade(&self, bytes: Vec<u8>) -> CallStageUpgrade {
        CallStageUpgrade::call(&self.contract).args(bytes)
    }

    pub fn deploy_upgrade(&self) -> CallDeployUpgrade {
        CallDeployUpgrade::call(&self.contract)
    }

    pub fn pause_precompiles(&self, paused_mask: u32) -> CallPausePrecompiles {
        CallPausePrecompiles::call(&self.contract).args_borsh(paused_mask)
    }

    pub fn resume_precompiles(&self, paused_mask: u32) -> CallResumePrecompiles {
        CallResumePrecompiles::call(&self.contract).args_borsh(paused_mask)
    }

    pub fn state_migration(&self) -> CallStateMigration {
        CallStateMigration::call(&self.contract)
    }

    pub fn mint_account(
        &self,
        address: Address,
        init_nonce: u64,
        init_balance: u64,
    ) -> CallMintAccount {
        CallMintAccount::call(&self.contract).args_borsh((address, init_nonce, init_balance))
    }

    pub fn set_paused_flags(&self, flags: PausedMask) -> CallSetPausedFlags {
        CallSetPausedFlags::call(&self.contract).args_borsh(flags)
    }

    pub fn set_key_manager(&self, args: RelayerKeyManagerArgs) -> CallSetKeyManager {
        CallSetKeyManager::call(&self.contract).args_json(args)
    }

    pub fn add_relayer_key(&self, key: RelayerKeyArgs) -> CallAddRelayerKey {
        CallAddRelayerKey::call(&self.contract).args_json(key)
    }

    pub fn remove_relayer_key(&self, key: RelayerKeyArgs) -> CallRemoveRelayerKey {
        CallRemoveRelayerKey::call(&self.contract).args_json(key)
    }

    pub fn pause_contract(&self) -> CallPauseContract {
        CallPauseContract::call(&self.contract)
    }

    pub fn resume_contract(&self) -> CallResumeContract {
        CallResumeContract::call(&self.contract)
    }

    pub fn set_fixed_gas(&self, cost: FixedGasArgs) -> CallSetFixedGas {
        CallSetFixedGas::call(&self.contract).args_borsh(cost)
    }

    pub fn set_silo_params(&self, params: Option<SiloParamsArgs>) -> CallSetSiloParams {
        CallSetSiloParams::call(&self.contract).args_borsh(params)
    }

    pub fn set_whitelist_status(&self, status: WhitelistStatusArgs) -> CallSetWhitelistStatus {
        CallSetWhitelistStatus::call(&self.contract).args_borsh(status)
    }

    pub fn add_entry_to_whitelist(&self, entry: WhitelistArgs) -> CallAddEntryToWhitelist {
        CallAddEntryToWhitelist::call(&self.contract).args_borsh(entry)
    }

    pub fn add_entry_to_whitelist_batch(
        &self,
        batch: Vec<WhitelistArgs>,
    ) -> CallAddEntryToWhitelistBatch {
        CallAddEntryToWhitelistBatch::call(&self.contract).args_borsh(batch)
    }

    pub fn remove_entry_from_whitelist(
        &self,
        entry: WhitelistArgs,
    ) -> CallRemoveEntryFromWhitelist {
        CallRemoveEntryFromWhitelist::call(&self.contract).args_borsh(entry)
    }

    pub fn set_erc20_metadata(&self, metadata: SetErc20MetadataArgs) -> CallSetErc20Metadata {
        CallSetErc20Metadata::call(&self.contract).args_json(metadata)
    }

    pub fn attach_full_access_key(&self, args: FullAccessKeyArgs) -> CallAttachFullAccessKey {
        CallAttachFullAccessKey::call(&self.contract).args_json(args)
    }

    pub fn set_owner(&self, account: &AccountId) -> CallSetOwner {
        CallSetOwner::call(&self.contract).args_borsh(account)
    }
}

/// View functions
impl EngineContract {
    pub fn ft_total_supply(&self) -> ViewFtTotalSupply {
        ViewFtTotalSupply::view(&self.contract)
    }

    pub fn ft_balance_of(&self, account_id: &AccountId) -> ViewFtBalanceOf {
        ViewFtBalanceOf::view(&self.contract).args_json(json!({ "account_id": account_id }))
    }

    pub fn ft_balances_of(&self, accounts: &Vec<AccountId>) -> ViewFtBalancesOf {
        ViewFtBalancesOf::view(&self.contract).args_borsh(accounts)
    }

    pub fn storage_balance_of(&self, account_id: &AccountId) -> ViewStorageBalanceOf {
        ViewStorageBalanceOf::view(&self.contract).args_json(json!({ "account_id": account_id }))
    }

    pub fn ft_metadata(&self) -> ViewFtMetadata {
        ViewFtMetadata::view(&self.contract)
    }

    pub fn get_version(&self) -> ViewVersion {
        ViewVersion::view(&self.contract)
    }

    pub fn get_owner(&self) -> ViewOwner {
        ViewOwner::view(&self.contract)
    }

    pub fn get_bridge_prover(&self) -> ViewBridgeProver {
        ViewBridgeProver::view(&self.contract)
    }

    pub fn get_chain_id(&self) -> ViewChainId {
        ViewChainId::view(&self.contract)
    }

    pub fn get_upgrade_index(&self) -> ViewUpgradeIndex {
        ViewUpgradeIndex::view(&self.contract)
    }

    pub fn get_paused_precompiles(&self) -> ViewPausedPrecompiles {
        ViewPausedPrecompiles::view(&self.contract)
    }

    pub fn get_block_hash(&self, block_height: u64) -> ViewBlockHash {
        ViewBlockHash::view(&self.contract).args_borsh(block_height)
    }

    pub fn get_code(&self, address: Address) -> ViewCode {
        ViewCode::view(&self.contract).args_borsh(address)
    }

    pub fn get_balance(&self, address: Address) -> ViewBalance {
        ViewBalance::view(&self.contract).args(address.as_bytes().to_vec())
    }

    pub fn get_nonce(&self, address: Address) -> ViewNonce {
        ViewNonce::view(&self.contract).args(address.as_bytes().to_vec())
    }

    pub fn get_storage_at(&self, address: Address, key: H256) -> ViewStorageAt {
        let raw_key = <H256 as Into<aurora_engine_types::types::RawH256>>::into(key);
        ViewStorageAt::view(&self.contract).args_borsh((address, raw_key))
    }

    pub fn get_view(
        &self,
        sender: Address,
        address: Address,
        amount: U256,
        input: Vec<u8>,
    ) -> ViewView {
        let mut raw_amount = [0u8; 32];
        amount.to_big_endian(&mut raw_amount);
        ViewView::view(&self.contract).args_borsh((sender, address, raw_amount, input))
    }

    pub fn is_used_proof(&self, proof: Proof) -> ViewIsUsedProof {
        ViewIsUsedProof::view(&self.contract).args_borsh(proof)
    }

    pub fn ft_total_eth_supply_on_aurora(&self) -> ViewFtTotalEthSupplyOnAurora {
        ViewFtTotalEthSupplyOnAurora::view(&self.contract)
    }

    pub fn ft_total_eth_supply_on_near(&self) -> ViewFtTotalEthSupplyOnNear {
        ViewFtTotalEthSupplyOnNear::view(&self.contract)
    }

    pub fn ft_balance_of_eth(&self, address: Address) -> ViewFtBalanceOfEth {
        ViewFtBalanceOfEth::view(&self.contract).args_borsh(address)
    }

    pub fn get_erc20_from_nep141(&self, account: AccountId) -> ViewErc20FromNep141 {
        ViewErc20FromNep141::view(&self.contract).args_borsh(account)
    }

    pub fn get_nep141_from_erc20(&self, address: Address) -> ViewNep141FromErc20 {
        ViewNep141FromErc20::view(&self.contract).args_borsh(address)
    }

    pub fn get_paused_flags(&self) -> ViewPausedFlags {
        ViewPausedFlags::view(&self.contract)
    }

    pub fn get_accounts_counter(&self) -> ViewAccountsCounter {
        ViewAccountsCounter::view(&self.contract)
    }

    pub fn get_eth_connector_contract_account(&self) -> ViewGetEthConnectorContractAccount {
        ViewGetEthConnectorContractAccount::view(&self.contract)
    }

    pub fn get_fixed_gas(&self) -> ViewGetFixedGas {
        ViewGetFixedGas::view(&self.contract)
    }

    pub fn get_silo_params(&self) -> ViewGetSiloParams {
        ViewGetSiloParams::view(&self.contract)
    }

    pub fn get_whitelist_status(&self, args: WhitelistKindArgs) -> ViewGetWhitelistStatus {
        ViewGetWhitelistStatus::view(&self.contract).args_borsh(args)
    }

    pub fn factory_get_wnear_address(&self) -> ViewFactoryWnearAddress {
        ViewFactoryWnearAddress::view(&self.contract)
    }

    pub fn get_erc20_metadata(&self, identifier: Erc20Identifier) -> ViewGetErc20Metadata {
        ViewGetErc20Metadata::view(&self.contract).args_json(identifier)
    }
}

#[derive(Debug, Clone)]
pub struct RawContract {
    inner: near_workspaces::Contract,
}

impl RawContract {
    pub fn from_contract(contract: near_workspaces::Contract) -> Self {
        Self { inner: contract }
    }

    pub fn call<F: AsRef<str>>(&self, function: F) -> CallTransaction {
        let call_tx = self.inner.call(function.as_ref());
        CallTransaction::new(call_tx)
    }

    pub fn view<F: AsRef<str>>(&self, function: F) -> ViewTransaction {
        let view_tx = self.inner.view(function.as_ref());
        ViewTransaction::new(view_tx)
    }

    pub fn id(&self) -> AccountId {
        self.inner.id().as_str().parse().unwrap()
    }
}
