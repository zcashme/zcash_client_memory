use std::num::NonZeroU8;

use incrementalmerkletree::frontier::Frontier;
use shardtree::{ShardTree, store::memory::MemoryShardStore};
use zcash_client_backend::data_api::{
    WalletWrite as _,
    chain::ChainState,
    testing::{
        AddressType, InitialChainState, TestBuilder, orchard::OrchardPoolTester,
        pool::ShieldedPoolTester,
    },
};
use zcash_primitives::block::BlockHash;
use zcash_protocol::{
    consensus::{NetworkUpgrade, Parameters},
    value::Zatoshis,
};

use crate::testing::{MemBlockCache, TestMemDbFactory};

#[test]
#[cfg(feature = "orchard")]
fn propose_v5_payment_to_orchard_receiver_is_rejected() {
    zcash_client_backend::data_api::testing::pool::propose_v5_payment_to_orchard_receiver_is_rejected::<_>(
        TestMemDbFactory::new(),
        MemBlockCache::new(),
    )
}

#[test]
#[ignore] // FIXME: pre-existing fee bug in put_tx_data overwrites fee on enhancement
#[cfg(feature = "orchard")]
fn orchard_to_ironwood_payment_reports_net_value_delta() {
    zcash_client_backend::data_api::testing::pool::orchard_to_ironwood_payment_reports_net_value_delta::<_>(
        TestMemDbFactory::new(),
        MemBlockCache::new(),
    )
}

#[test]
#[ignore] // FIXME: pre-existing todo!() in store_decrypted_tx for Incoming outputs
#[cfg(feature = "orchard")]
fn orchard_to_ironwood_self_migration_reports_fee_only_delta() {
    zcash_client_backend::data_api::testing::pool::orchard_to_ironwood_self_migration_reports_fee_only_delta::<_>(
        TestMemDbFactory::new(),
        MemBlockCache::new(),
    )
}

#[test]
#[cfg(feature = "orchard")]
fn truncate_wallet_saved_before_ironwood() {
    let mut st = TestBuilder::new()
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    let dfvk = OrchardPoolTester::test_account_fvk(&st);
    let value = Zatoshis::const_from_u64(10_000);
    let (h, _, _) = st.generate_next_block(&dfvk, AddressType::DefaultExternal, value);
    for _ in 0..5 {
        st.generate_next_block(&dfvk, AddressType::DefaultExternal, value);
    }
    st.scan_cached_blocks(h, 6);

    // Wallets saved before Ironwood support load with an empty Ironwood tree.
    st.wallet_mut().ironwood_tree = ShardTree::new(MemoryShardStore::empty(), 100);

    assert_eq!(st.wallet_mut().truncate_to_height(h + 2).unwrap(), h + 2);
}

#[test]
#[cfg(feature = "orchard")]
fn truncate_below_first_scanned_block_with_empty_ironwood_birthday() {
    let mut st = TestBuilder::new()
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_initial_chain_state(|rng, network| {
            let birthday_height = network.activation_height(NetworkUpgrade::Nu5).unwrap() + 1000;
            let (_, sapling_initial_tree) =
                Frontier::random_with_prior_subtree_roots(rng, 100, NonZeroU8::new(16).unwrap());
            let (_, orchard_initial_tree) =
                Frontier::random_with_prior_subtree_roots(rng, 100, NonZeroU8::new(16).unwrap());

            InitialChainState {
                chain_state: ChainState::new(
                    birthday_height - 1,
                    BlockHash([5; 32]),
                    sapling_initial_tree,
                    orchard_initial_tree,
                    Frontier::empty(),
                ),
                prior_sapling_roots: vec![],
                prior_orchard_roots: vec![],
            }
        })
        .with_account_having_current_birthday()
        .build();

    let birthday = st.test_account().unwrap().birthday().height();
    let dfvk = OrchardPoolTester::test_account_fvk(&st);
    let value = Zatoshis::const_from_u64(10_000);
    for _ in 0..15 {
        st.generate_next_block(&dfvk, AddressType::DefaultExternal, value);
    }

    // Scan above the birthday first, then truncate below the first scanned block.
    st.scan_cached_blocks(birthday + 10, 5);

    assert_eq!(
        st.wallet_mut().truncate_to_height(birthday + 5).unwrap(),
        birthday - 1
    );
}
