use std::path::Path;

use zeroize::Zeroize;

use crate::node::NodeError;

/// Reprezentuje tożsamość węzła PQC pobraną prosto z nexum-cli vault'a (magazynu w C).
pub struct PQCIdentity {
    pub falcon_pk: Vec<u8>,   // Załadowane z nexum-cli / vault (T_FALCON_PK)
    pub falcon_sk: zeroize::Zeroizing<Vec<u8>>,   // Klucz tajny dla podpisywania Consensus Vote — Zeroize chroni przed memory dump
    pub frodo_prekeys: Vec<Vec<u8>>, // Pula prekey'ów KEM do asynchronicznych DM
}

// Tag types as per nexum-cli/src/vault.c
const T_KEM_SK: u16 = 0x0004;
const T_FALCON_SK: u16 = 0x0006;
const T_FALCON_PK: u16 = 0x0007;
const T_PREKEY_B: u16 = 0x000A;

impl PQCIdentity {
    /// Odtwarza zawartość tożsamości parsując wyeksportowane z `nexum-cli` zrzuty skrytki TLV.
    /// Jeśli plik nie istnieje, system zwróci błąd inicjalizacji (Węzeł nie wstanie bez kluczy PQC).
    pub fn load_from_vault(vault_path: &Path) -> Result<Self, NodeError> {
        if !vault_path.exists() {
            return Err(NodeError::IdentityError(format!("Plik vault.c nie istnieje w sciezce: {:?}", vault_path)));
        }

        let bytes = std::fs::read(vault_path).map_err(|_| NodeError::IdentityError("Nie mozna wczytac wyeksportowanego pliku z Nexum CLI".into()))?;
        
        let mut falcon_pk = vec![];
        let mut falcon_sk = vec![];
        let mut frodo_prekeys = vec![];

        let mut i = 0;
        while i + 4 <= bytes.len() {
            // Type-Length-Value format according to `write_tlv()` in C
            let t = u16::from_le_bytes([bytes[i], bytes[i+1]]);
            let n = u16::from_le_bytes([bytes[i+2], bytes[i+3]]) as usize;
            i += 4;

            if i + n > bytes.len() {
                break;
            }

            let val = &bytes[i..i+n];
            match t {
                T_FALCON_SK => falcon_sk = val.to_vec(),
                T_FALCON_PK => falcon_pk = val.to_vec(),
                T_KEM_SK => { /* Używane w przypadku operacji decryption/transport */ },
                T_PREKEY_B => frodo_prekeys.push(val.to_vec()), // Parsowanie wygenerowanych wczesniej kluczy bundle w 'prekeys.c'
                _ => {}
            }
            i += n;
        }

        if falcon_sk.is_empty() || falcon_pk.is_empty() {
             return Err(NodeError::IdentityError("Plik Vault nie zawiera obowiazkowych sygnatur Falcon PQC!".into()));
        }

        Ok(Self {
            falcon_pk,
            falcon_sk: zeroize::Zeroizing::new(falcon_sk),
            frodo_prekeys,
        })
    }
}
