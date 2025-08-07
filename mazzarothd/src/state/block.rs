use consensus::{block_header::ConsensusHeader, traits::ConsensusHeaderStorage, types::BlockKey};
use database::rocksdb_no_batch::RocksDbStorage;
use mvm::{core::storage::DbStorage, models::block::Block};
use utils::error::{Error, Result};

pub struct BlockStorage {
    db: RocksDbStorage,
}

impl BlockStorage {
    pub fn new(db: &str) -> anyhow::Result<Self> {
        let db = RocksDbStorage::new(db)?;
        Ok(Self { db })
    }

    pub fn get_block(&self, key: &BlockKey) -> anyhow::Result<Option<Block>> {
        let block = self.db.get_data(key)?;
        Ok(block)
    }

    pub fn set_block(&self, key: &BlockKey, block: &Block) -> anyhow::Result<()> {
        self.db.set_data(key, block)?;
        Ok(())
    }

    pub fn has_block(&self, key: &BlockKey) -> anyhow::Result<bool> {
        let exists = self.db.has_data(key)?;
        Ok(exists)
    }
}

impl ConsensusHeaderStorage for BlockStorage {
    fn get_consensus_header(&self, key: &BlockKey) -> Result<ConsensusHeader> {
        let block = self.get_block(key)?;
        let block = block.ok_or_else(|| Error::BlockNotFound {
            key: key.to_string(),
        })?;
        Ok(block.inner.header)
    }
}
