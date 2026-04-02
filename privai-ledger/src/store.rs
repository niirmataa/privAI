use std::fs;
use std::path::{Path, PathBuf};

use privai_chain::DEFAULT_CHAIN_ID;
use rocksdb::{DB, Options, ColumnFamilyDescriptor};

use crate::error::StoreError;
use crate::state::LedgerSnapshot;

/// Column Family names for RocksDB
pub const CF_LEDGER_STATE: &str = "ledger_state";
pub const CF_BLOCKS: &str = "blocks";
pub const CF_NOTES: &str = "notes";
pub const CF_NULLIFIERS: &str = "nullifiers";
pub const CF_TICKET_NULLIFIERS: &str = "ticket_nullifiers";
pub const CF_QCS: &str = "qcs";

pub trait LedgerStore {
    fn load(&self) -> Result<Option<LedgerSnapshot>, StoreError>;
    fn save(&mut self, snapshot: &LedgerSnapshot) -> Result<(), StoreError>;
}

#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    snapshot: Option<LedgerSnapshot>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self { snapshot: None }
    }
}

impl LedgerStore for MemoryStore {
    fn load(&self) -> Result<Option<LedgerSnapshot>, StoreError> {
        Ok(self.snapshot.clone())
    }

    fn save(&mut self, snapshot: &LedgerSnapshot) -> Result<(), StoreError> {
        self.snapshot = Some(snapshot.clone());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FileSystemStore {
    root: PathBuf,
}

impl FileSystemStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("ledger-state.json")
    }

    pub fn ensure_initialized(&mut self) -> Result<(), StoreError> {
        if self.load()?.is_none() {
            self.save(&LedgerSnapshot::genesis(DEFAULT_CHAIN_ID))?;
        }
        Ok(())
    }
}

impl LedgerStore for FileSystemStore {
    fn load(&self) -> Result<Option<LedgerSnapshot>, StoreError> {
        let state_path = self.state_path();
        if !state_path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(state_path)?;
        Ok(Some(serde_json::from_slice(&bytes)?))
    }

    fn save(&mut self, snapshot: &LedgerSnapshot) -> Result<(), StoreError> {
        fs::create_dir_all(&self.root)?;
        let state_path = self.state_path();
        let bytes = serde_json::to_vec_pretty(snapshot)?;

        // Atomic write: temp + rename (atomic na większości FS w tym samym mount point)
        // Chroni przed korupcją stanu przy crash mid-write.
        let tmp_path = state_path.with_extension("tmp");
        fs::write(&tmp_path, bytes)?;
        fs::rename(&tmp_path, &state_path)?;
        Ok(())
    }
}

/// RocksDB-backed persistent store for production use.
/// Uses column families for different data types.
pub struct RocksDBStore {
    db: DB,
}

impl RocksDBStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        
        let mut cf_opts = Options::default();
        cf_opts.set_max_write_buffer_number(16);
        cf_opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
        cf_opts.set_target_file_size_base(64 * 1024 * 1024); // 64MB
        cf_opts.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB
        
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_LEDGER_STATE, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_BLOCKS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_NOTES, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_NULLIFIERS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_TICKET_NULLIFIERS, cf_opts.clone()),
            ColumnFamilyDescriptor::new(CF_QCS, cf_opts.clone()),
        ];

        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let db = DB::open_cf_descriptors(&db_opts, path, cfs)
            .map_err(|e| StoreError::RocksDB(e.to_string()))?;

        Ok(Self { db })
    }

    pub fn ensure_initialized(&self) -> Result<(), StoreError> {
        let cf = self.db.cf_handle(CF_LEDGER_STATE)
            .ok_or_else(|| StoreError::RocksDB("ledger_state CF not found".into()))?;
        
        if self.db.get_cf(&cf, b"snapshot").map_err(|e| StoreError::RocksDB(e.to_string()))?.is_none() {
            let genesis = LedgerSnapshot::genesis(DEFAULT_CHAIN_ID);
            let bytes = serde_json::to_vec(&genesis)?;
            self.db.put_cf(&cf, b"snapshot", &bytes)
                .map_err(|e| StoreError::RocksDB(e.to_string()))?;
        }
        Ok(())
    }
}

impl LedgerStore for RocksDBStore {
    fn load(&self) -> Result<Option<LedgerSnapshot>, StoreError> {
        let cf = self.db.cf_handle(CF_LEDGER_STATE)
            .ok_or_else(|| StoreError::RocksDB("ledger_state CF not found".into()))?;
        
        match self.db.get_cf(&cf, b"snapshot").map_err(|e| StoreError::RocksDB(e.to_string()))? {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    fn save(&mut self, snapshot: &LedgerSnapshot) -> Result<(), StoreError> {
        let cf = self.db.cf_handle(CF_LEDGER_STATE)
            .ok_or_else(|| StoreError::RocksDB("ledger_state CF not found".into()))?;
        
        let bytes = serde_json::to_vec(snapshot)?;
        self.db.put_cf(&cf, b"snapshot", &bytes)
            .map_err(|e| StoreError::RocksDB(e.to_string()))?;
        
        // Sync to disk for durability
        self.db.flush().map_err(|e| StoreError::RocksDB(e.to_string()))?;
        Ok(())
    }
}
