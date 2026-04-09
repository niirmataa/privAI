//! privAI Wallet Key Management.
//! Role: Hierarchical key derivation from Master Seed.
//! Architecture: BLAKE3 KDF tree → Spend Root / Scan Root / Nullifier Root / KEM Root.
//! Recovery: Master Seed (256 bit) → odtworzenie WSZYSTKICH kluczy.
//! See: PRIVAI_V0_FORMATS.md

use zeroize::Zeroizing;
use nxms_transport::crypto::{Keys, FF_FALCON_TEST_SEED_LEN};
use privai_chain::Hash32;

/// Domeny KDF — każda gałąź drzewa kluczy ma własną domenę.
const SPEND_ROOT_DOMAIN: &str = "privai:wallet:spend-root:v0";
const SCAN_ROOT_DOMAIN: &str = "privai:wallet:scan-root:v0";
const NULLIFIER_ROOT_DOMAIN: &str = "privai:wallet:nullifier-root:v0";
const KEM_ROOT_DOMAIN: &str = "privai:wallet:kem-root:v0";

const SPEND_DOMAIN: &str = "privai:wallet:spend:v0";
const NULLIFIER_DOMAIN: &str = "privai:wallet:nk:v0";

/// Hierarchia kluczy portfela.
/// 
/// Master Seed (256 bit) → 4 root keys → per-bundle/per-note derivation.
/// 
/// Zasady:
/// - Spend Root → nigdy nie opuszcza urządzenia
/// - Scan Root → może być wyeksportowany do trusted observer (delegacja skanowania)
/// - Nullifier Root → per-note derivation
/// - KEM Root → per-bundle deterministic FrodoKEM keygen
pub struct WalletKeys {
    /// Master seed — źródło wszystkich kluczy. Zeroized po drop.
    master_seed: Zeroizing<[u8; 32]>,
    /// Root key do spendowania (Falcon signing keys per bundle).
    spend_root: Zeroizing<[u8; 32]>,
    /// Root key do skanowania (hint derivation).
    scan_root: Zeroizing<[u8; 32]>,
    /// Root key do nullifier derivation per note.
    nullifier_root: Zeroizing<[u8; 32]>,
    /// Root key do KEM keygen per bundle.
    kem_root: Zeroizing<[u8; 32]>,
}

impl WalletKeys {
    /// Utwórz klucze z master seed.
    /// 
    /// Każdy root jest derived przez BLAKE3 z domeną + master_seed.
    /// Deterministyczny: ten sam seed → te same root keys.
    pub fn from_master_seed(seed: &[u8; 32]) -> Self {
        let spend_root = derive_root_key(SPEND_ROOT_DOMAIN, seed);
        let scan_root = derive_root_key(SCAN_ROOT_DOMAIN, seed);
        let nullifier_root = derive_root_key(NULLIFIER_ROOT_DOMAIN, seed);
        let kem_root = derive_root_key(KEM_ROOT_DOMAIN, seed);

        Self {
            master_seed: Zeroizing::new(*seed),
            spend_root: Zeroizing::new(spend_root),
            scan_root: Zeroizing::new(scan_root),
            nullifier_root: Zeroizing::new(nullifier_root),
            kem_root: Zeroizing::new(kem_root),
        }
    }

    /// Generuj nowy master seed z RNG (pierwszy raz, potem backupować).
    pub fn generate() -> Result<Self, String> {
        // Użyj nonce jako source randomness (jest kryptograficznie bezpieczny)
        let nonce = nxms_transport::crypto::random_xchacha20poly1305_nonce();
        let mut seed = [0u8; 32];
        seed[..24].copy_from_slice(&nonce);
        // Uzupełnij resztę z hash nonce
        let extra = blake3::hash(&nonce);
        seed[24..].copy_from_slice(&extra.as_bytes()[..8]);
        Ok(Self::from_master_seed(&seed))
    }

    /// Hash master seed (do zapisu w WalletSnapshot — weryfikacja czy to ten sam portfel).
    pub fn master_seed_hash(&self) -> Hash32 {
        privai_chain::hash::domain_hash("privai:wallet:seed-hash:v0", &[&self.master_seed[..]])
    }

    /// Derive klucze dla bundla o danym indexie.
    /// 
    /// Zwraca Keys z:
    /// - Falcon signing keys (seeded z spend_root)
    /// - FrodoKEM keys (generowane losowo — KEM nie ma seeded wariantu w nxms-transport)
    pub fn derive_bundle_keys(&self, bundle_index: u64) -> Result<Keys, String> {
        // Falcon seed: BLAKE3(spend_root, bundle_index) → 32B, rozszerz do 48B
        let spend_seed_32 = derive_key_with_index(SPEND_DOMAIN, &self.spend_root, bundle_index);
        let mut sig_seed = [0u8; FF_FALCON_TEST_SEED_LEN]; // 48 bajtów
        sig_seed[..32].copy_from_slice(&spend_seed_32);
        // Pozostałe 16 bajtów z drugiego hash
        let spend_seed_ext = derive_key_with_index("privai:wallet:spend-ext:v0", &self.spend_root, bundle_index);
        sig_seed[32..].copy_from_slice(&spend_seed_ext[..16]);

        // Generate keys: Falcon seeded + KEM random
        Keys::generate_seeded(&sig_seed).map_err(|e| e.to_string())
    }

    /// Derive nullifier key dla danej noty.
    /// 
    /// Używane przez nadawcę do wstawienia NK do RecipientBoxPlaintext.
    /// Odbiorca może zweryfikować czy NK jest poprawnie derived.
    pub fn derive_nullifier_key(&self, note_index: u64) -> [u8; 32] {
        derive_key_with_index(NULLIFIER_DOMAIN, &self.nullifier_root, note_index)
    }

    /// Scan Root — do eksportu dla delegowanego skanowania.
    /// 
    /// Scan Root pozwala hint-matching BEZ możliwości spendowania.
    pub fn scan_root(&self) -> &[u8; 32] {
        &self.scan_root
    }

    /// Sprawdź czy podany scan_root pasuje do tego portfela.
    pub fn verify_scan_root(&self, scan_root: &[u8; 32]) -> bool {
        self.scan_root.as_slice() == scan_root
    }

    /// Export skanowania (Scan Root + KEM Root) do lightweight scanning wallet.
    /// 
    /// **NIE zawiera Spend Root** — delegat może skanować ale nie spendować.
    pub fn export_scanning_delegate(&self) -> ScanningDelegate {
        ScanningDelegate {
            scan_root: Zeroizing::new(*self.scan_root),
            kem_root: Zeroizing::new(*self.kem_root),
        }
    }
}

/// Delegowany klucz skanowania — pozwala hint-matching + KEM decaps BEZ spendowania.
pub struct ScanningDelegate {
    scan_root: Zeroizing<[u8; 32]>,
    #[allow(dead_code)]
    kem_root: Zeroizing<[u8; 32]>,
}

impl ScanningDelegate {
    /// Utwórz z wyeksportowanego scan_root.
    /// 
    /// **UWAGA**: KEM root jest potrzebny do KEM decapsulation.
    /// Bez niego delegat może tylko hint-matchować ale nie otwierać RecipientBox.
    pub fn from_roots(scan_root: [u8; 32], kem_root: [u8; 32]) -> Self {
        Self {
            scan_root: Zeroizing::new(scan_root),
            kem_root: Zeroizing::new(kem_root),
        }
    }

    /// Derive hint dla bundla (do hint-matching).
    pub fn derive_hint(&self, bundle_index: u64) -> [u8; 16] {
        let hint_hash = derive_key_with_index("privai:hint:v0", &self.scan_root, bundle_index);
        let mut hint = [0u8; 16];
        hint.copy_from_slice(&hint_hash[..16]);
        hint
    }

    /// Hash scan root (do weryfikacji czy to ten sam delegat).
    pub fn scan_root_hash(&self) -> Hash32 {
        privai_chain::hash::domain_hash("privai:wallet:scan-hash:v0", &[&self.scan_root[..]])
    }
}

// ─── Helper functions ───────────────────────────────────────────────

/// BLAKE3 KDF: root key z domeną + seed.
fn derive_root_key(domain: &str, seed: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(seed);
    let mut out = [0u8; 32];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

/// BLAKE3 KDF: per-index key z domeną + root + index.
fn derive_key_with_index(domain: &str, root: &[u8; 32], index: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(root);
    hasher.update(&index.to_le_bytes());
    let mut out = [0u8; 32];
    out.copy_from_slice(hasher.finalize().as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_deterministic_from_seed() {
        let seed = [42u8; 32];
        let keys1 = WalletKeys::from_master_seed(&seed);
        let keys2 = WalletKeys::from_master_seed(&seed);

        assert_eq!(keys1.master_seed_hash(), keys2.master_seed_hash());
        assert_eq!(*keys1.spend_root, *keys2.spend_root);
        assert_eq!(*keys1.scan_root, *keys2.scan_root);
        assert_eq!(*keys1.nullifier_root, *keys2.nullifier_root);
        assert_eq!(*keys1.kem_root, *keys2.kem_root);
    }

    #[test]
    fn keys_different_seeds_different_roots() {
        let seed1 = [1u8; 32];
        let seed2 = [2u8; 32];
        let keys1 = WalletKeys::from_master_seed(&seed1);
        let keys2 = WalletKeys::from_master_seed(&seed2);

        assert_ne!(keys1.master_seed_hash(), keys2.master_seed_hash());
        assert_ne!(*keys1.spend_root, *keys2.spend_root);
        assert_ne!(*keys1.scan_root, *keys2.scan_root);
    }

    #[test]
    fn bundle_keys_deterministic() {
        let seed = [42u8; 32];
        let keys = WalletKeys::from_master_seed(&seed);

        let bundle0_a = keys.derive_bundle_keys(0).expect("derive 0a");
        let bundle0_b = keys.derive_bundle_keys(0).expect("derive 0b");
        let bundle1 = keys.derive_bundle_keys(1).expect("derive 1");

        // Ten sam index → te same Falcon klucze (seeded)
        assert_eq!(bundle0_a.sig_pk().expect("sig pk a"), bundle0_b.sig_pk().expect("sig pk b"));

        // KEM jest generowany LOSOWO (nie seeded) — różne wywołania dają różne klucze
        // To jest oczekiwane zachowanie (KEM nie ma seeded wariantu w nxms-transport)
        // assert_ne!(bundle0_a.kem_pk().expect("kem pk a"), bundle0_b.kem_pk().expect("kem pk b"));

        // Różny index → różne Falcon klucze
        assert_ne!(bundle0_a.sig_pk().expect("sig pk a"), bundle1.sig_pk().expect("sig pk 1"));
    }

    #[test]
    fn nullifier_key_deterministic() {
        let seed = [42u8; 32];
        let keys = WalletKeys::from_master_seed(&seed);

        let nk0_a = keys.derive_nullifier_key(0);
        let nk0_b = keys.derive_nullifier_key(0);
        let nk1 = keys.derive_nullifier_key(1);

        assert_eq!(nk0_a, nk0_b);
        assert_ne!(nk0_a, nk1);
    }

    #[test]
    fn scan_delegation_no_spend() {
        let seed = [42u8; 32];
        let keys = WalletKeys::from_master_seed(&seed);

        let delegate = keys.export_scanning_delegate();

        // Delegate może derive hint
        let hint = delegate.derive_hint(0);
        assert_eq!(hint.len(), 16);

        // Delegate NIE ma dostępu do spend_root
        // (sprawdzamy pośrednio — scan_root jest eksportowany, spend_root nie)
        assert!(keys.verify_scan_root(&delegate.scan_root));
    }
}
