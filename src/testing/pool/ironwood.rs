use std::convert::Infallible;
use std::num::{NonZeroU8, NonZeroU32};

use incrementalmerkletree::{Hashable, Level, frontier::Frontier};
use shardtree::{ShardTree, store::memory::MemoryShardStore};
use zcash_client_backend::{
    data_api::{
        Account as _, IRONWOOD_SHARD_HEIGHT, WalletCommitmentTrees as _, WalletRead as _,
        WalletWrite as _,
        chain::{ChainState, CommitmentTreeRoot},
        testing::{
            AddressType, InitialChainState, IronwoodFvk, TestBuilder, orchard::OrchardPoolTester,
            pool::ShieldedPoolTester, single_output_change_strategy,
        },
        wallet::{ConfirmationsPolicy, input_selection::GreedyInputSelector},
    },
    fees::StandardFeeRule,
    wallet::OvkPolicy,
    zip321::{Payment, TransactionRequest},
};
use zcash_primitives::block::BlockHash;
use zcash_protocol::{
    ShieldedPool,
    consensus::{BlockHeight, NetworkUpgrade, Parameters},
    local_consensus::LocalNetwork,
    value::Zatoshis,
};

use crate::{
    proto::memwallet as proto,
    testing::{MemBlockCache, TestMemDbFactory},
};

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

#[test]
#[cfg(feature = "orchard")]
fn scan_progress_counts_unscanned_ironwood_actions() {
    let mut st = TestBuilder::new()
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    // Blocks whose only shielded activity is Ironwood.
    let fvk = IronwoodFvk(OrchardPoolTester::test_account_fvk(&st));
    let value = Zatoshis::const_from_u64(10_000);
    let (h, _, _) = st.generate_next_block(&fvk, AddressType::DefaultExternal, value);
    for _ in 0..4 {
        st.generate_next_block(&fvk, AddressType::DefaultExternal, value);
    }

    // The next Ironwood subtree ends above these blocks.
    st.wallet_mut()
        .put_ironwood_subtree_roots(
            1,
            &[CommitmentTreeRoot::from_parts(
                h + 1000,
                orchard::tree::MerkleHashOrchard::empty_root(Level::from(IRONWOOD_SHARD_HEIGHT)),
            )],
        )
        .unwrap();

    // Scan only part of the range up to the chain tip.
    st.wallet_mut().update_chain_tip(h + 4).unwrap();
    st.scan_cached_blocks(h, 2);

    let summary = st
        .wallet()
        .get_wallet_summary(ConfirmationsPolicy::MIN)
        .unwrap()
        .unwrap();
    let scan_progress = summary.progress().scan();
    assert!(scan_progress.denominator() > &0);
    assert!(scan_progress.numerator() < scan_progress.denominator());
}

#[test]
#[cfg(feature = "orchard")]
fn anchor_height_respects_lagging_ironwood_tree() {
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

    // Make the Ironwood tree lag the other trees.
    let lag_height = h + 2;
    assert!(
        st.wallet_mut()
            .ironwood_tree
            .truncate_to_checkpoint(&lag_height)
            .unwrap()
    );

    let (_, anchor_height) = st
        .wallet()
        .get_target_and_anchor_heights(NonZeroU32::MIN)
        .unwrap()
        .unwrap();
    assert_eq!(anchor_height, lag_height);
}

#[test]
#[cfg(feature = "orchard")]
fn get_transaction_keeps_ironwood_bundle_without_cached_expiry() {
    // A network on which NU6.3, the version 6 transaction format, is active from height 100_000.
    let activation = BlockHeight::from_u32(100_000);
    let ironwood_active_network = LocalNetwork {
        nu6: Some(activation),
        nu6_1: Some(activation),
        nu6_2: Some(activation),
        nu6_3: Some(activation),
        ..TestBuilder::<(), ()>::DEFAULT_NETWORK
    };
    let mut st = TestBuilder::new()
        .with_network(ironwood_active_network)
        .with_data_store_factory(TestMemDbFactory::new())
        .with_block_cache(MemBlockCache::new())
        .with_account_from_sapling_activation(BlockHash([0; 32]))
        .build();

    let dfvk = OrchardPoolTester::test_account_fvk(&st);
    let (h, _, _) = st.generate_next_block(
        &dfvk,
        AddressType::DefaultExternal,
        Zatoshis::const_from_u64(60_000),
    );
    st.scan_cached_blocks(h, 1);

    // Post-NU6.3, a payment to an Orchard receiver is routed through the Ironwood pool.
    let to = OrchardPoolTester::sk_default_address(&OrchardPoolTester::sk(&[0xf5; 32]));
    let account = st.test_account().unwrap();
    let account_id = account.account().id();
    let usk = account.usk().clone();
    let request = TransactionRequest::new(vec![Payment::without_memo(
        to.to_zcash_address(st.network()),
        Zatoshis::const_from_u64(10_000),
    )])
    .unwrap();
    let change_strategy =
        single_output_change_strategy(StandardFeeRule::Zip317, None, ShieldedPool::Orchard);
    let proposal = st
        .propose_transfer(
            account_id,
            &GreedyInputSelector::new(),
            &change_strategy,
            request,
            ConfirmationsPolicy::MIN,
        )
        .unwrap();
    let txid = *st
        .create_proposed_transactions::<Infallible, _, Infallible, _>(
            &usk,
            OvkPolicy::Sender,
            &proposal,
        )
        .unwrap()
        .first();
    let tx = st.wallet().get_transaction(txid).unwrap().unwrap();
    assert!(tx.ironwood_bundle().is_some());

    // Forget the cached expiry height of the unmined transaction.
    let entry = st.wallet().tx_table.get(&txid).unwrap().clone();
    let mut proto_entry = proto::TransactionEntry::from(entry);
    proto_entry.expiry_height = None;
    st.wallet_mut()
        .tx_table
        .0
        .insert(txid, proto_entry.try_into().unwrap());

    let tx = st.wallet().get_transaction(txid).unwrap().unwrap();
    assert_eq!(tx.txid(), txid);
    assert!(tx.ironwood_bundle().is_some());
}
