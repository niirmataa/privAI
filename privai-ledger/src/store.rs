use std::fs;
use std::path::{Path, PathBuf};

use privai_chain::DEFAULT_CHAIN_ID;

use crate::error::StoreError;
use crate::state::LedgerSnapshot;

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
        fs::write(state_path, bytes)?;
        Ok(())
    }
}
