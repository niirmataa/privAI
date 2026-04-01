use std::fs;
use std::path::{Path, PathBuf};

use crate::error::WalletStoreError;
use crate::state::{ManagedBundle, OwnedNoteRecord, WalletSnapshot};
use privai_chain::{BundleId, Hash32};
use serde::{Deserialize, Serialize};

pub trait WalletStore {
    fn load(&self) -> Result<Option<WalletSnapshot>, WalletStoreError>;
    fn save(&mut self, snapshot: &WalletSnapshot) -> Result<(), WalletStoreError>;
}

#[derive(Clone, Debug, Default)]
pub struct MemoryWalletStore {
    snapshot: Option<WalletSnapshot>,
}

impl MemoryWalletStore {
    pub fn new() -> Self {
        Self { snapshot: None }
    }
}

impl WalletStore for MemoryWalletStore {
    fn load(&self) -> Result<Option<WalletSnapshot>, WalletStoreError> {
        Ok(self.snapshot.clone())
    }

    fn save(&mut self, snapshot: &WalletSnapshot) -> Result<(), WalletStoreError> {
        self.snapshot = Some(snapshot.clone());
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct FileSystemWalletStore {
    root: PathBuf,
}

impl FileSystemWalletStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn state_path(&self) -> PathBuf {
        self.root.join("wallet-state.json")
    }
}

impl WalletStore for FileSystemWalletStore {
    fn load(&self) -> Result<Option<WalletSnapshot>, WalletStoreError> {
        let state_path = self.state_path();
        if !state_path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(state_path)?;
        let persisted: PersistedWalletSnapshot = serde_json::from_slice(&bytes)?;
        Ok(Some(persisted.into_snapshot()))
    }

    fn save(&mut self, snapshot: &WalletSnapshot) -> Result<(), WalletStoreError> {
        fs::create_dir_all(&self.root)?;
        let state_path = self.state_path();
        let bytes = serde_json::to_vec_pretty(&PersistedWalletSnapshot::from_snapshot(snapshot))?;
        fs::write(state_path, bytes)?;
        Ok(())
    }
}

use crate::small_payments_rail::RailContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedWalletSnapshot {
    bundles: Vec<PersistedBundleEntry>,
    owned_notes: Vec<PersistedOwnedNoteEntry>,
    #[serde(default)]
    rail_context: Option<RailContext>,
}

impl PersistedWalletSnapshot {
    fn from_snapshot(snapshot: &WalletSnapshot) -> Self {
        Self {
            bundles: snapshot
                .bundles
                .iter()
                .map(|(bundle_id, managed_bundle)| PersistedBundleEntry {
                    bundle_id: *bundle_id,
                    managed_bundle: managed_bundle.clone(),
                })
                .collect(),
            owned_notes: snapshot
                .owned_notes
                .iter()
                .map(|(note_commit, record)| PersistedOwnedNoteEntry {
                    note_commit: *note_commit,
                    record: record.clone(),
                })
                .collect(),
            rail_context: snapshot.rail_context.clone(),
        }
    }

    fn into_snapshot(self) -> WalletSnapshot {
        WalletSnapshot {
            bundles: self
                .bundles
                .into_iter()
                .map(|entry| (entry.bundle_id, entry.managed_bundle))
                .collect(),
            owned_notes: self
                .owned_notes
                .into_iter()
                .map(|entry| (entry.note_commit, entry.record))
                .collect(),
            rail_context: self.rail_context,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedBundleEntry {
    bundle_id: BundleId,
    managed_bundle: ManagedBundle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PersistedOwnedNoteEntry {
    note_commit: Hash32,
    record: OwnedNoteRecord,
}
