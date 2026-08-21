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