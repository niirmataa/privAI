use crate::node::NodeError;
use std::path::Path;

/// Reprezentuje tożsamość węzła PQC pobraną prosto z nexum-cli vault'a (magazynu w C).
pub struct PQCIdentity {
    pub falcon_pk: Vec<u8>, // Załadowane z nexum-cli / vault (T_FALCON_PK)
    pub falcon_sk: zeroize::Zeroizing<Vec<u8>>, // Klucz tajny dla podpisywania Consensus Vote — Zeroize chroni przed memory dump
    pub kem_pk: Vec<u8>, // Klucz publiczny FrodoKEM dla warstwy transportowej
    pub kem_sk: zeroize::Zeroizing<Vec<u8>>, // Klucz tajny FrodoKEM
    pub frodo_prekeys: Vec<Vec<u8>>, // Pula prekey'ów KEM do asynchronicznych DM
}

// Current vault TLV tags from nexum-cli/src/vault.c.
const T_FALCON_SK: u16 = 3;
const T_FALCON_PK: u16 = 4;
const T_KEM_SK: u16 = 5;
const T_KEM_PK: u16 = 6;
// Legacy optional bundle-prekey tag kept for compatibility with older exports.
const T_PREKEY_B: u16 = 0x000A;

#[derive(Debug, Default)]
struct ParsedVaultIdentity {
    falcon_pk: Vec<u8>,
    falcon_sk: Vec<u8>,
    kem_pk: Vec<u8>,
    kem_sk: Vec<u8>,
    frodo_prekeys: Vec<Vec<u8>>,
}

fn parse_vault_identity_tlv(bytes: &[u8]) -> Result<ParsedVaultIdentity, NodeError> {
    let mut parsed = ParsedVaultIdentity::default();
    let mut i = 0usize;

    while i + 6 <= bytes.len() {
        let t = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
        let n =
            u32::from_le_bytes([bytes[i + 2], bytes[i + 3], bytes[i + 4], bytes[i + 5]]) as usize;
        i += 6;

        let end = i
            .checked_add(n)
            .ok_or_else(|| NodeError::IdentityError("Vault TLV length overflow".into()))?;
        if end > bytes.len() {
            return Err(NodeError::IdentityError(
                "Vault TLV entry exceeds input size".into(),
            ));
        }

        let val = &bytes[i..end];
        match t {
            T_FALCON_SK => parsed.falcon_sk = val.to_vec(),
            T_FALCON_PK => parsed.falcon_pk = val.to_vec(),
            T_KEM_SK => parsed.kem_sk = val.to_vec(),
            T_KEM_PK => parsed.kem_pk = val.to_vec(),
            T_PREKEY_B => parsed.frodo_prekeys.push(val.to_vec()),
            _ => {}
        }
        i = end;
    }

    Ok(parsed)
}

impl PQCIdentity {
    /// Odtwarza zawartość tożsamości parsując wyeksportowane z `nexum-cli` zrzuty skrytki TLV.
    /// Jeśli plik nie istnieje, system zwróci błąd inicjalizacji (Węzeł nie wstanie bez kluczy PQC).
    pub fn load_from_vault(vault_path: &Path) -> Result<Self, NodeError> {
        if !vault_path.exists() {
            return Err(NodeError::IdentityError(format!(
                "Plik vault.c nie istnieje w sciezce: {:?}",
                vault_path
            )));
        }

        let bytes = std::fs::read(vault_path).map_err(|_| {
            NodeError::IdentityError("Nie mozna wczytac wyeksportowanego pliku z Nexum CLI".into())
        })?;
        let parsed = parse_vault_identity_tlv(&bytes)?;

        if parsed.falcon_sk.is_empty() || parsed.falcon_pk.is_empty() {
            return Err(NodeError::IdentityError(
                "Plik Vault nie zawiera obowiazkowych sygnatur Falcon PQC!".into(),
            ));
        }

        Ok(Self {
            falcon_pk: parsed.falcon_pk,
            falcon_sk: zeroize::Zeroizing::new(parsed.falcon_sk),
            kem_pk: parsed.kem_pk,
            kem_sk: zeroize::Zeroizing::new(parsed.kem_sk),
            frodo_prekeys: parsed.frodo_prekeys,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_vault_identity_tlv, T_FALCON_PK, T_FALCON_SK, T_KEM_PK, T_KEM_SK};

    fn push_tlv(out: &mut Vec<u8>, tag: u16, value: &[u8]) {
        out.extend_from_slice(&tag.to_le_bytes());
        out.extend_from_slice(&(value.len() as u32).to_le_bytes());
        out.extend_from_slice(value);
    }

    #[test]
    fn parser_matches_current_vault_header_and_tags() {
        let mut bytes = Vec::new();
        push_tlv(&mut bytes, T_FALCON_SK, b"falcon-secret");
        push_tlv(&mut bytes, T_FALCON_PK, b"falcon-public");
        push_tlv(&mut bytes, T_KEM_SK, b"kem-secret");
        push_tlv(&mut bytes, T_KEM_PK, b"kem-public");

        let parsed = parse_vault_identity_tlv(&bytes).expect("parse current vault tlv");

        assert_eq!(parsed.falcon_sk, b"falcon-secret");
        assert_eq!(parsed.falcon_pk, b"falcon-public");
        assert_eq!(parsed.kem_sk, b"kem-secret");
        assert_eq!(parsed.kem_pk, b"kem-public");
    }
}
