use std::convert::Infallible;

use incrementalmerkletree::Address;
use shardtree::{ShardTree, error::ShardTreeError, store::memory::MemoryShardStore, store::ShardStore as _};
#[cfg(feature = "orchard")]
use zcash_client_backend::data_api::{IRONWOOD_SHARD_HEIGHT, ORCHARD_SHARD_HEIGHT};
use zcash_client_backend::data_api::{
    SAPLING_SHARD_HEIGHT, WalletCommitmentTrees, chain::CommitmentTreeRoot,
};
use zcash_protocol::consensus::{self, BlockHeight};

use crate::MemoryWalletDb;

impl<P: consensus::Parameters> WalletCommitmentTrees for MemoryWalletDb<P> {
    type Error = Infallible;
    type SaplingShardStore<'a> = MemoryShardStore<sapling::Node, BlockHeight>;

    fn with_sapling_tree_mut<F, A, E>(&mut self, mut callback: F) -> Result<A, E>
    where
        for<'a> F: FnMut(
            &'a mut ShardTree<
                Self::SaplingShardStore<'a>,
                { sapling::NOTE_COMMITMENT_TREE_DEPTH },
                SAPLING_SHARD_HEIGHT,
            >,
        ) -> Result<A, E>,
        E: From<ShardTreeError<Infallible>>,
    {
        tracing::debug!("with_sapling_tree_mut");
        callback(&mut self.sapling_tree)
    }

    fn put_sapling_subtree_roots(
        &mut self,
        start_index: u64,
        roots: &[CommitmentTreeRoot<sapling::Node>],
    ) -> Result<(), ShardTreeError<Self::Error>> {
        tracing::debug!("put_sapling_subtree_roots");
        self.with_sapling_tree_mut(|t| {
            for (root, i) in roots.iter().zip(0u64..) {
                let root_addr = Address::from_parts(SAPLING_SHARD_HEIGHT.into(), start_index + i);
                t.insert(root_addr, *root.root_hash())?;
            }
            Ok::<_, ShardTreeError<Self::Error>>(())
        })?;

        // store the end block heights for each shard as well
        for (root, i) in roots.iter().zip(0u64..) {
            let root_addr = Address::from_parts(SAPLING_SHARD_HEIGHT.into(), start_index + i);
            self.sapling_tree_shard_end_heights
                .insert(root_addr, root.subtree_end_height());
        }

        Ok(())
    }

    #[cfg(feature = "orchard")]
    type OrchardShardStore<'a> = MemoryShardStore<orchard::tree::MerkleHashOrchard, BlockHeight>;

    #[cfg(feature = "orchard")]
    fn with_orchard_tree_mut<F, A, E>(&mut self, mut callback: F) -> Result<A, E>
    where
        for<'a> F: FnMut(
            &'a mut ShardTree<
                Self::OrchardShardStore<'a>,
                { ORCHARD_SHARD_HEIGHT * 2 },
                ORCHARD_SHARD_HEIGHT,
            >,
        ) -> Result<A, E>,
        E: From<ShardTreeError<Self::Error>>,
    {
        tracing::debug!("with_orchard_tree_mut");
        callback(&mut self.orchard_tree)
    }

    /// Returns the stored root hash of the completed Sapling subtree with the given index,
    /// or `Ok(None)` if no root is recorded for that subtree.
    fn get_sapling_subtree_root(
        &mut self,
        index: u64,
    ) -> Result<Option<sapling::Node>, ShardTreeError<Self::Error>> {
        let root_addr = Address::from_parts(SAPLING_SHARD_HEIGHT.into(), index);
        Ok(
            if let Some(shard) = self
                .sapling_tree
                .store()
                .get_shard(root_addr)
                .map_err(ShardTreeError::Storage)?
            {
                // Only a completed subtree has a root.
                if shard.max_position() == Some(root_addr.max_position()) {
                    shard.root_hash(root_addr.position_range_end()).ok()
                } else {
                    None
                }
            } else {
                None
            },
        )
    }

    /// Adds a sequence of note commitment tree subtree roots to the data store.
    #[cfg(feature = "orchard")]
    fn put_orchard_subtree_roots(
        &mut self,
        start_index: u64,
        roots: &[CommitmentTreeRoot<orchard::tree::MerkleHashOrchard>],
    ) -> Result<(), ShardTreeError<Self::Error>> {
        tracing::debug!("put_orchard_subtree_roots");
        self.with_orchard_tree_mut(|t| {
            for (root, i) in roots.iter().zip(0u64..) {
                let root_addr = Address::from_parts(ORCHARD_SHARD_HEIGHT.into(), start_index + i);
                t.insert(root_addr, *root.root_hash())?;
            }
            Ok::<_, ShardTreeError<Self::Error>>(())
        })?;

        // store the end block heights for each shard as well
        for (root, i) in roots.iter().zip(0u64..) {
            let root_addr = Address::from_parts(ORCHARD_SHARD_HEIGHT.into(), start_index + i);
            self.orchard_tree_shard_end_heights
                .insert(root_addr, root.subtree_end_height());
        }

        Ok(())
    }

    /// Returns the stored root hash of the completed Orchard subtree with the given index,
    /// or `Ok(None)` if no root is recorded for that subtree.
    #[cfg(feature = "orchard")]
    fn get_orchard_subtree_root(
        &mut self,
        index: u64,
    ) -> Result<Option<orchard::tree::MerkleHashOrchard>, ShardTreeError<Self::Error>> {
        let root_addr = Address::from_parts(ORCHARD_SHARD_HEIGHT.into(), index);
        Ok(
            if let Some(shard) = self
                .orchard_tree
                .store()
                .get_shard(root_addr)
                .map_err(ShardTreeError::Storage)?
            {
                // Only a completed subtree has a root.
                if shard.max_position() == Some(root_addr.max_position()) {
                    shard.root_hash(root_addr.position_range_end()).ok()
                } else {
                    None
                }
            } else {
                None
            },
        )
    }

    #[cfg(feature = "orchard")]
    fn with_ironwood_tree_mut<F, A, E>(&mut self, mut callback: F) -> Result<Option<A>, E>
    where
        for<'a> F: FnMut(
            &'a mut ShardTree<
                Self::OrchardShardStore<'a>,
                { IRONWOOD_SHARD_HEIGHT * 2 },
                IRONWOOD_SHARD_HEIGHT,
            >,
        ) -> Result<A, E>,
        E: From<ShardTreeError<Self::Error>>,
    {
        tracing::debug!("with_ironwood_tree_mut");
        callback(&mut self.ironwood_tree).map(Some)
    }

    /// Adds a sequence of Ironwood note commitment tree subtree roots to the data store.
    #[cfg(feature = "orchard")]
    fn put_ironwood_subtree_roots(
        &mut self,
        start_index: u64,
        roots: &[CommitmentTreeRoot<orchard::tree::MerkleHashOrchard>],
    ) -> Result<(), ShardTreeError<Self::Error>> {
        tracing::debug!("put_ironwood_subtree_roots");
        self.with_ironwood_tree_mut(|t| {
            for (root, i) in roots.iter().zip(0u64..) {
                let root_addr = Address::from_parts(IRONWOOD_SHARD_HEIGHT.into(), start_index + i);
                t.insert(root_addr, *root.root_hash())?;
            }
            Ok::<_, ShardTreeError<Self::Error>>(())
        })?;

        // store the end block heights for each shard as well
        for (root, i) in roots.iter().zip(0u64..) {
            let root_addr = Address::from_parts(IRONWOOD_SHARD_HEIGHT.into(), start_index + i);
            self.ironwood_tree_shard_end_heights
                .insert(root_addr, root.subtree_end_height());
        }

        Ok(())
    }

    /// Returns the stored root hash of the completed Ironwood subtree with the given index,
    /// or `Ok(None)` if no root is recorded for that subtree.
    #[cfg(feature = "orchard")]
    fn get_ironwood_subtree_root(
        &mut self,
        index: u64,
    ) -> Result<Option<orchard::tree::MerkleHashOrchard>, ShardTreeError<Self::Error>> {
        let root_addr = Address::from_parts(IRONWOOD_SHARD_HEIGHT.into(), index);
        Ok(
            if let Some(shard) = self
                .ironwood_tree
                .store()
                .get_shard(root_addr)
                .map_err(ShardTreeError::Storage)?
            {
                // Only a completed subtree has a root.
                if shard.max_position() == Some(root_addr.max_position()) {
                    shard.root_hash(root_addr.position_range_end()).ok()
                } else {
                    None
                }
            } else {
                None
            },
        )
    }
}

pub(crate) mod serialization {
    use std::io::Cursor;

    use incrementalmerkletree::{Address, Level};
    use shardtree::{
        LocatedPrunableTree, ShardTree,
        store::{Checkpoint, ShardStore, memory::MemoryShardStore},
    };
    use zcash_client_backend::serialization::shardtree::{read_shard, write_shard};
    use zcash_protocol::consensus::BlockHeight;

    use crate::{Error, proto::memwallet as proto};

    pub(crate) fn tree_to_protobuf<
        H: Clone
            + incrementalmerkletree::Hashable
            + PartialEq
            + zcash_primitives::merkle_tree::HashSer,
        const DEPTH: u8,
        const SHARD_HEIGHT: u8,
    >(
        tree: &ShardTree<MemoryShardStore<H, BlockHeight>, DEPTH, SHARD_HEIGHT>,
    ) -> Result<Option<crate::proto::memwallet::ShardTree>, Error> {
        use crate::proto::memwallet::{ShardTree, TreeCheckpoint, TreeShard};

        let mut cap_bytes = Vec::new();
        write_shard(&mut cap_bytes, &tree.store().get_cap()?)?;

        let shards = tree
            .store()
            .get_shard_roots()?
            .iter()
            .map(|shard_root| {
                let shard = tree.store().get_shard(*shard_root)?.unwrap();

                let mut shard_data = Vec::new();
                write_shard(&mut shard_data, shard.root())?;

                Ok(TreeShard {
                    shard_index: shard_root.index(),
                    shard_data,
                })
            })
            .collect::<Result<Vec<TreeShard>, Error>>()?;

        let mut checkpoints = Vec::new();
        tree.store()
            .for_each_checkpoint(usize::MAX, |id, checkpoint| {
                checkpoints.push(TreeCheckpoint {
                    checkpoint_id: (*id).into(),
                    position: match checkpoint.tree_state() {
                        shardtree::store::TreeState::Empty => 0,
                        shardtree::store::TreeState::AtPosition(position) => position.into(),
                    },
                });
                Ok(())
            })
            .ok();

        Ok(Some(ShardTree {
            cap: cap_bytes,
            shards,
            checkpoints,
        }))
    }

    pub(crate) fn tree_from_protobuf<
        H: Clone
            + incrementalmerkletree::Hashable
            + PartialEq
            + zcash_primitives::merkle_tree::HashSer,
        const DEPTH: u8,
        const SHARD_HEIGHT: u8,
    >(
        proto_tree: proto::ShardTree,
        max_checkpoints: usize,
        shard_root_level: Level,
    ) -> Result<ShardTree<MemoryShardStore<H, BlockHeight>, DEPTH, SHARD_HEIGHT>, Error> {
        let mut tree = ShardTree::new(MemoryShardStore::empty(), max_checkpoints);

        let cap = read_shard(Cursor::new(&proto_tree.cap))?;
        tree.store_mut().put_cap(cap)?;

        for proto_shard in proto_tree.shards {
            let shard_root = Address::from_parts(shard_root_level, proto_shard.shard_index);
            let shard_tree = read_shard(&mut Cursor::new(proto_shard.shard_data))?;
            let shard = LocatedPrunableTree::from_parts(shard_root, shard_tree)
                .map_err(|_| Error::Other("shard persistence invalid".to_string()))?;
            tree.store_mut().put_shard(shard)?;
        }

        for proto_checkpoint in proto_tree.checkpoints {
            tree.store_mut().add_checkpoint(
                BlockHeight::from(proto_checkpoint.checkpoint_id),
                Checkpoint::at_position(proto_checkpoint.position.into()),
            )?;
        }

        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use incrementalmerkletree::{Hashable, Level, Retention};
    use zcash_client_backend::data_api::{
        SAPLING_SHARD_HEIGHT, WalletCommitmentTrees, chain::CommitmentTreeRoot,
    };
    use zcash_protocol::consensus::{BlockHeight, Network};

    use crate::MemoryWalletDb;

    #[cfg(feature = "orchard")]
    use zcash_client_backend::data_api::{IRONWOOD_SHARD_HEIGHT, ORCHARD_SHARD_HEIGHT};

    #[test]
    fn sapling_subtree_root() {
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        let root = sapling::Node::empty_root(Level::from(SAPLING_SHARD_HEIGHT));
        wallet
            .put_sapling_subtree_roots(
                0,
                &[CommitmentTreeRoot::from_parts(
                    BlockHeight::from(500_000),
                    root,
                )],
            )
            .unwrap();
        assert_eq!(wallet.get_sapling_subtree_root(0).unwrap(), Some(root));
        assert_eq!(wallet.get_sapling_subtree_root(1).unwrap(), None);

        // An incomplete subtree has no root.
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        wallet
            .sapling_tree
            .append(sapling::Node::empty_leaf(), Retention::Ephemeral)
            .unwrap();
        assert_eq!(wallet.get_sapling_subtree_root(0).unwrap(), None);
    }

    #[test]
    #[cfg(feature = "orchard")]
    fn orchard_subtree_root() {
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        let root = orchard::tree::MerkleHashOrchard::empty_root(Level::from(ORCHARD_SHARD_HEIGHT));
        wallet
            .put_orchard_subtree_roots(
                0,
                &[CommitmentTreeRoot::from_parts(
                    BlockHeight::from(500_000),
                    root,
                )],
            )
            .unwrap();
        assert_eq!(wallet.get_orchard_subtree_root(0).unwrap(), Some(root));
        assert_eq!(wallet.get_orchard_subtree_root(1).unwrap(), None);

        // An incomplete subtree has no root.
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        wallet
            .orchard_tree
            .append(
                orchard::tree::MerkleHashOrchard::empty_leaf(),
                Retention::Ephemeral,
            )
            .unwrap();
        assert_eq!(wallet.get_orchard_subtree_root(0).unwrap(), None);
    }

    #[test]
    #[cfg(feature = "orchard")]
    fn ironwood_subtree_root() {
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        let root = orchard::tree::MerkleHashOrchard::empty_root(Level::from(IRONWOOD_SHARD_HEIGHT));
        wallet
            .put_ironwood_subtree_roots(
                0,
                &[CommitmentTreeRoot::from_parts(
                    BlockHeight::from(500_000),
                    root,
                )],
            )
            .unwrap();
        assert_eq!(wallet.get_ironwood_subtree_root(0).unwrap(), Some(root));
        assert_eq!(wallet.get_ironwood_subtree_root(1).unwrap(), None);

        // An incomplete subtree has no root.
        let mut wallet = MemoryWalletDb::new(Network::TestNetwork, 100);
        wallet
            .ironwood_tree
            .append(
                orchard::tree::MerkleHashOrchard::empty_leaf(),
                Retention::Ephemeral,
            )
            .unwrap();
        assert_eq!(wallet.get_ironwood_subtree_root(0).unwrap(), None);
    }
}
