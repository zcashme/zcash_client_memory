use crate::testing;

use zcash_client_backend::data_api::testing::orchard::OrchardPoolTester;
use zcash_client_backend::data_api::testing::sapling::SaplingPoolTester;
use zcash_client_backend::data_api::{
    Account as _, InputSource as _, WalletRead as _,
    testing::{AddressType, IronwoodFvk, TestBuilder, pool::ShieldedPoolTester},
    wallet::{ConfirmationsPolicy, TargetHeight, input_selection::LockFilter},
};
use zcash_primitives::block::BlockHash;
use zcash_protocol::{ShieldedPool, value::Zatoshis};

use crate::testing::{MemBlockCache, TestMemDbFactory};

#[test]
fn send_single_step_proposed_transfer() {
    testing::pool::send_single_step_proposed_transfer::<OrchardPoolTester>()
}

#[test]
fn scan_full_block_detects_outputs() {
    testing::pool::scan_full_block_detects_outputs::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
#[cfg(feature = "transparent-inputs")]
fn send_multi_step_proposed_transfer() {
    testing::pool::send_multi_step_proposed_transfer::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
#[cfg(feature = "transparent-inputs")]
fn proposal_fails_if_not_all_ephemeral_outputs_consumed() {
    testing::pool::proposal_fails_if_not_all_ephemeral_outputs_consumed::<OrchardPoolTester>()
}

#[test]
#[allow(deprecated)]
fn create_to_address_fails_on_incorrect_usk() {
    testing::pool::create_to_address_fails_on_incorrect_usk::<OrchardPoolTester>()
}

#[test]
#[allow(deprecated)]
fn proposal_fails_with_no_blocks() {
    testing::pool::proposal_fails_with_no_blocks::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn spend_fails_on_unverified_notes() {
    testing::pool::spend_fails_on_unverified_notes::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn spend_fails_on_locked_notes() {
    testing::pool::spend_fails_on_locked_notes::<OrchardPoolTester>()
}

#[test]
#[cfg(feature = "orchard")]
fn single_note_selection_honors_lock_tier_preference() {
    testing::pool::single_note_selection_honors_lock_tier_preference::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn ovk_policy_prevents_recovery_from_chain() {
    testing::pool::ovk_policy_prevents_recovery_from_chain::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn spend_succeeds_to_t_addr_zero_change() {
    testing::pool::spend_succeeds_to_t_addr_zero_change::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn change_note_spends_succeed() {
    testing::pool::change_note_spends_succeed::<OrchardPoolTester>()
}

#[test]
fn external_address_change_spends_detected_in_restore_from_seed() {
    testing::pool::external_address_change_spends_detected_in_restore_from_seed::<OrchardPoolTester>(
    )
}

#[test]
#[ignore] // FIXME: #1316 This requires support for dust outputs.
fn zip317_spend() {
    testing::pool::zip317_spend::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
#[cfg(feature = "transparent-inputs")]
fn shield_transparent() {
    testing::pool::shield_transparent::<OrchardPoolTester>()
}

#[test]
fn birthday_in_anchor_shard() {
    testing::pool::birthday_in_anchor_shard::<OrchardPoolTester>()
}

#[test]
fn checkpoint_gaps() {
    testing::pool::checkpoint_gaps::<OrchardPoolTester>()
}

#[test]
#[cfg(feature = "orchard")]
fn pool_crossing_required() {
    testing::pool::pool_crossing_required::<OrchardPoolTester, SaplingPoolTester>()
}

#[test]
#[cfg(feature = "orchard")]
fn fully_funded_fully_private() {
    testing::pool::fully_funded_fully_private::<OrchardPoolTester, SaplingPoolTester>()
}

#[test]
#[cfg(all(feature = "orchard", feature = "transparent-inputs"))]
#[ignore] //FIXME
fn fully_funded_send_to_t() {
    testing::pool::fully_funded_send_to_t::<OrchardPoolTester, SaplingPoolTester>()
}

#[test]
#[cfg(feature = "orchard")]
fn multi_pool_checkpoint() {
    testing::pool::multi_pool_checkpoint::<OrchardPoolTester, SaplingPoolTester>()
}

#[test]
#[cfg(feature = "orchard")]
fn multi_pool_checkpoints_with_pruning() {
    testing::pool::multi_pool_checkpoints_with_pruning::<OrchardPoolTester, SaplingPoolTester>()
}

#[test]
fn valid_chain_states() {
    testing::pool::valid_chain_states::<OrchardPoolTester>()
}

#[test]
fn invalid_chain_cache_disconnected() {
    testing::pool::invalid_chain_cache_disconnected::<OrchardPoolTester>()
}

#[test]
fn data_db_truncation() {
    testing::pool::data_db_truncation::<OrchardPoolTester>()
}

#[test]
fn scan_cached_blocks_allows_blocks_out_of_order() {
    testing::pool::scan_cached_blocks_allows_blocks_out_of_order::<OrchardPoolTester>()
}

#[test]
fn scan_cached_blocks_finds_received_notes() {
    testing::pool::scan_cached_blocks_finds_received_notes::<OrchardPoolTester>()
}

#[test]
fn scan_cached_blocks_finds_change_notes() {
    testing::pool::scan_cached_blocks_finds_change_notes::<OrchardPoolTester>()
}

#[test]
fn scan_cached_blocks_detects_spends_out_of_order() {
    testing::pool::scan_cached_blocks_detects_spends_out_of_order::<OrchardPoolTester>()
}

#[test]
#[ignore] //FIXME
fn receive_two_notes_with_same_value() {
    testing::pool::receive_two_notes_with_same_value::<OrchardPoolTester>()
}

#[test]
fn select_single_spendable_note_finds_covering_note_after_smaller_notes() {
    let mut st = TestBuilder::new()
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    let dfvk = OrchardPoolTester::test_account_fvk(&st);
    let (h, _, _) = st.generate_next_block(
        &dfvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(10_000),
    );
    st.generate_next_block(
        &dfvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(10_000),
    );
    st.generate_next_block(
        &dfvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(40_000),
    );
    st.scan_cached_blocks(h, 3);

    let account_id = st.test_account().unwrap().account().id();
    let target_height = TargetHeight::from(st.wallet().chain_height().unwrap().unwrap() + 1);
    let notes = st
        .wallet()
        .select_single_spendable_note(
            account_id,
            Zatoshis::const_from_u64(15_000),
            &[ShieldedPool::Orchard],
            target_height,
            ConfirmationsPolicy::MIN,
            &[],
            LockFilter::Unfiltered,
        )
        .unwrap();

    assert_eq!(notes.orchard().len(), 1);
    assert_eq!(
        notes.orchard()[0].note_value().unwrap(),
        Zatoshis::const_from_u64(40_000)
    );
}

#[test]
fn select_single_spendable_note_uses_first_pool_in_sources() {
    let mut st = TestBuilder::new()
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    let orchard_fvk = OrchardPoolTester::test_account_fvk(&st);
    let ironwood_fvk = IronwoodFvk(orchard_fvk.clone());
    let (h, _, _) = st.generate_next_block(
        &orchard_fvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(40_000),
    );
    st.generate_next_block(
        &ironwood_fvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(40_000),
    );
    st.scan_cached_blocks(h, 2);

    let account_id = st.test_account().unwrap().account().id();
    let target_height = TargetHeight::from(st.wallet().chain_height().unwrap().unwrap() + 1);
    let notes = st
        .wallet()
        .select_single_spendable_note(
            account_id,
            Zatoshis::const_from_u64(15_000),
            &[ShieldedPool::Ironwood, ShieldedPool::Orchard],
            target_height,
            ConfirmationsPolicy::MIN,
            &[],
            LockFilter::Unfiltered,
        )
        .unwrap();

    assert_eq!(notes.ironwood().len(), 1);
    assert!(notes.orchard().is_empty());
}
