use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fmt;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_void};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::ptr;
use zeroize::{Zeroize, Zeroizing};

const NXMS_KEM_ID: &str = "FrodoKEM-640-SHAKE";
const NXMS_SIG_ID: &str = "Falcon-1024-CT";

pub const NXMS_CONTEXT_ID_LEN: usize = 16;
const FF_FALCON_N: usize = 1 << 10;

// Max sizes (from nxms_ms_transport.h)
const NXMS_MAX_PAYLOAD: usize = 16 * 1024 * 1024;
const NXMS_MIN_KEM_BYTES: usize = 64;
const NXMS_MAX_KEM_PK_LEN: usize = 32768;
const NXMS_MAX_KEM_CT_LEN: usize = 32768;
const NXMS_MAX_KEM_SK_LEN: usize = 65536;
const NXMS_MAX_SIG_SK_LEN: usize = 65536;
const NXMS_MAX_SIG_PK_LEN: usize = 65536;
const NXMS_NONCE_LEN: usize = 24;
const NXMS_TAG_LEN: usize = 32;
const FF_FALCON_SIG_MAX: usize = 4096;
pub const FF_FALCON_TEST_SEED_LEN: usize = 48;
pub const XCHACHA20POLY1305_KEY_LEN: usize = 32;
pub const XCHACHA20POLY1305_NONCE_LEN: usize = 24;
pub const XCHACHA20POLY1305_TAG_LEN: usize = 16;

#[repr(C)]
pub struct ff_kem_keys_t {
    pub alg: [u8; 32],
    pub pk: *mut u8,
    pub pk_len: usize,
    pub sk: *mut u8,
    pub sk_len: usize,
}

#[repr(C)]
struct ff_falcon_prepared_sk_t {
    magic: u32,
    logn: u32,
    f: [i8; FF_FALCON_N],
    g: [i8; FF_FALCON_N],
    f_cap: [i8; FF_FALCON_N],
    g_cap: [i8; FF_FALCON_N],
}

#[repr(C)]
struct nxms_ms_signer_ctx {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn ff_kem_keygen(alg: *const i8, out: *mut ff_kem_keys_t) -> i32;
    fn ff_kem_encaps(
        alg: *const i8,
        pk: *const u8,
        pk_len: usize,
        ct: *mut *mut u8,
        ct_len: *mut usize,
        ss: *mut *mut u8,
        ss_len: *mut usize,
    ) -> i32;
    fn ff_kem_decaps(
        alg: *const i8,
        sk: *const u8,
        sk_len: usize,
        ct: *const u8,
        ct_len: usize,
        ss: *mut *mut u8,
        ss_len: *mut usize,
    ) -> i32;
    fn ff_kem_free(k: *mut ff_kem_keys_t);

    fn ff_falcon_keygen(sk: *mut u8, sk_len: *mut usize, pk: *mut u8, pk_len: *mut usize) -> i32;
    fn ff_falcon_keygen_seeded(
        seed: *const u8,
        seed_len: usize,
        sk: *mut u8,
        sk_len: *mut usize,
        pk: *mut u8,
        pk_len: *mut usize,
    ) -> i32;
    fn ff_falcon_sign_ct(
        sk: *const u8,
        sk_len: usize,
        msg: *const u8,
        msg_len: usize,
        sig: *mut u8,
        sig_len: *mut usize,
    ) -> i32;
    fn ff_falcon_sign_ct_seeded(
        seed: *const u8,
        seed_len: usize,
        sk: *const u8,
        sk_len: usize,
        msg: *const u8,
        msg_len: usize,
        sig: *mut u8,
        sig_len: *mut usize,
    ) -> i32;
    fn ff_falcon_prepare_sk(
        sk: *const u8,
        sk_len: usize,
        prepared: *mut ff_falcon_prepared_sk_t,
    ) -> i32;
    fn ff_falcon_clear_prepared_sk(prepared: *mut ff_falcon_prepared_sk_t);
    fn ff_falcon_sign_ct_prepared(
        prepared: *const ff_falcon_prepared_sk_t,
        msg: *const u8,
        msg_len: usize,
        sig: *mut u8,
        sig_len: *mut usize,
    ) -> i32;
    fn ff_falcon_sign_ct_prepared_seeded(
        seed: *const u8,
        seed_len: usize,
        prepared: *const ff_falcon_prepared_sk_t,
        msg: *const u8,
        msg_len: usize,
        sig: *mut u8,
        sig_len: *mut usize,
    ) -> i32;
    fn ff_falcon_verify(
        pk: *const u8,
        pk_len: usize,
        msg: *const u8,
        msg_len: usize,
        sig: *const u8,
        sig_len: usize,
    ) -> i32;

    fn nxms_ms_encrypt_packet(
        sender_id: *const c_char,
        to_id: *const c_char,
        msg_type: *const c_char,
        escrow_id_raw: *const u8,
        seq: u64,
        recipient_pk_kem: *const u8,
        recipient_pk_kem_len: usize,
        sender_sk_sig: *const u8,
        sender_sk_sig_len: usize,
        plaintext: *const u8,
        plaintext_len: usize,
        kem_ct: *mut *mut u8,
        kem_ct_len: *mut usize,
        nonce: *mut *mut u8,
        nonce_len: *mut usize,
        ciphertext: *mut *mut u8,
        ciphertext_len: *mut usize,
        tag: *mut *mut u8,
        tag_len: *mut usize,
        sig: *mut *mut u8,
        sig_len: *mut usize,
    ) -> i32;
    fn nxms_ms_signer_ctx_new(
        sender_sk_sig: *const u8,
        sender_sk_sig_len: usize,
    ) -> *mut nxms_ms_signer_ctx;
    fn nxms_ms_signer_ctx_free(ctx: *mut nxms_ms_signer_ctx);
    fn nxms_ms_encrypt_packet_with_signer(
        sender_id: *const c_char,
        to_id: *const c_char,
        msg_type: *const c_char,
        escrow_id_raw: *const u8,
        seq: u64,
        recipient_pk_kem: *const u8,
        recipient_pk_kem_len: usize,
        signer: *const nxms_ms_signer_ctx,
        plaintext: *const u8,
        plaintext_len: usize,
        kem_ct: *mut *mut u8,
        kem_ct_len: *mut usize,
        nonce: *mut *mut u8,
        nonce_len: *mut usize,
        ciphertext: *mut *mut u8,
        ciphertext_len: *mut usize,
        tag: *mut *mut u8,
        tag_len: *mut usize,
        sig: *mut *mut u8,
        sig_len: *mut usize,
    ) -> i32;

    fn nxms_ms_verify_decrypt(
        sender_id: *const c_char,
        to_id: *const c_char,
        msg_type: *const c_char,
        escrow_id_raw: *const u8,
        seq: u64,
        kem_ct: *const u8,
        kem_ct_len: usize,
        nonce: *const u8,
        nonce_len: usize,
        ciphertext: *const u8,
        ciphertext_len: usize,
        tag: *const u8,
        tag_len: usize,
        sig: *const u8,
        sig_len: usize,
        recipient_sk_kem: *const u8,
        recipient_sk_kem_len: usize,
        sender_pk_sig: *const u8,
        sender_pk_sig_len: usize,
        out_plain: *mut *mut u8,
        out_plain_len: *mut usize,
    ) -> i32;
    fn nxms_ms_free(ptr: *mut c_void);
    fn nxms_ms_free_secure(ptr: *mut c_void, len: usize);

    fn sodium_init() -> i32;
    fn randombytes_buf(buf: *mut c_void, size: usize);
    fn crypto_aead_xchacha20poly1305_ietf_encrypt(
        c: *mut u8,
        clen_p: *mut u64,
        m: *const u8,
        mlen: u64,
        ad: *const u8,
        adlen: u64,
        nsec: *const u8,
        npub: *const u8,
        k: *const u8,
    ) -> i32;
    fn crypto_aead_xchacha20poly1305_ietf_decrypt(
        m: *mut u8,
        mlen_p: *mut u64,
        nsec: *mut u8,
        c: *const u8,
        clen: u64,
        ad: *const u8,
        adlen: u64,
        npub: *const u8,
        k: *const u8,
    ) -> i32;
}

struct KemKeysGuard {
    inner: ff_kem_keys_t,
}

impl KemKeysGuard {
    fn new() -> Self {
        Self {
            inner: ff_kem_keys_t {
                alg: [0u8; 32],
                pk: ptr::null_mut(),
                pk_len: 0,
                sk: ptr::null_mut(),
                sk_len: 0,
            },
        }
    }

    fn as_mut_ptr(&mut self) -> *mut ff_kem_keys_t {
        &mut self.inner
    }

    fn as_ref(&self) -> &ff_kem_keys_t {
        &self.inner
    }
}

impl Drop for KemKeysGuard {
    fn drop(&mut self) {
        unsafe { ff_kem_free(&mut self.inner) };
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SecretB64(String);

impl SecretB64 {
    fn new(value: String) -> Self {
        Self(value)
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for SecretB64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl Drop for SecretB64 {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Keys {
    kem_sk_b64: SecretB64,
    pub kem_pk_b64: String,
    sig_sk_b64: SecretB64,
    pub sig_pk_b64: String,
}

impl Keys {
    pub fn generate() -> Result<Self> {
        Self::generate_internal(None)
    }

    pub fn generate_seeded(sig_seed: &[u8; FF_FALCON_TEST_SEED_LEN]) -> Result<Self> {
        Self::generate_internal(Some(sig_seed))
    }

    fn generate_internal(sig_seed: Option<&[u8; FF_FALCON_TEST_SEED_LEN]>) -> Result<Self> {
        // FrodoKEM via liboqs wrapper
        let alg = CString::new(NXMS_KEM_ID)?;
        let mut k = KemKeysGuard::new();
        let rc = unsafe { ff_kem_keygen(alg.as_ptr(), k.as_mut_ptr()) };
        if rc != 0 {
            return Err(anyhow!("ff_kem_keygen failed rc={}", rc));
        }
        let (pk, mut sk) = unsafe { copy_kem_key_material(k.as_ref()) }?;

        // Falcon
        let (mut sig_sk, sig_pk) = falcon_keygen_internal(sig_seed)?;

        let kem_sk_b64 = B64.encode(&sk);
        sk.zeroize();
        let sig_sk_b64 = B64.encode(&sig_sk);
        sig_sk.zeroize();

        Ok(Self {
            kem_sk_b64: SecretB64::new(kem_sk_b64),
            kem_pk_b64: B64.encode(pk),
            sig_sk_b64: SecretB64::new(sig_sk_b64),
            sig_pk_b64: B64.encode(sig_pk),
        })
    }

    pub fn read_json(path: &std::path::Path) -> Result<Self> {
        let data = Zeroizing::new(std::fs::read(path)?);
        Ok(serde_json::from_slice(data.as_slice())?)
    }

    pub fn write_json(&self, path: &str) -> Result<()> {
        let data = Zeroizing::new(serde_json::to_vec_pretty(self)?);
        std::fs::write(path, data.as_slice())?;
        #[cfg(unix)]
        {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn kem_pk(&self) -> Result<Vec<u8>> {
        decode_key_b64(
            &self.kem_pk_b64,
            "kem_pk",
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_PK_LEN,
        )
    }
    fn decode_kem_sk(&self) -> Result<Vec<u8>> {
        decode_key_b64(
            self.kem_sk_b64.as_str(),
            "kem_sk",
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_SK_LEN,
        )
    }
    #[deprecated(note = "use kem_sk_zeroizing() to avoid retaining secret key bytes in memory")]
    pub fn kem_sk(&self) -> Result<Vec<u8>> {
        self.decode_kem_sk()
    }
    pub fn kem_sk_zeroizing(&self) -> Result<Zeroizing<Vec<u8>>> {
        Ok(Zeroizing::new(self.decode_kem_sk()?))
    }
    pub fn sig_pk(&self) -> Result<Vec<u8>> {
        decode_key_b64(&self.sig_pk_b64, "sig_pk", 1, NXMS_MAX_SIG_PK_LEN)
    }
    fn decode_sig_sk(&self) -> Result<Vec<u8>> {
        decode_key_b64(self.sig_sk_b64.as_str(), "sig_sk", 1, NXMS_MAX_SIG_SK_LEN)
    }
    #[deprecated(note = "use sig_sk_zeroizing() to avoid retaining secret key bytes in memory")]
    pub fn sig_sk(&self) -> Result<Vec<u8>> {
        self.decode_sig_sk()
    }
    pub fn sig_sk_zeroizing(&self) -> Result<Zeroizing<Vec<u8>>> {
        Ok(Zeroizing::new(self.decode_sig_sk()?))
    }
}

#[derive(Clone, Debug)]
pub struct SealedPacket {
    pub kem_ct_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
    pub tag_b64: String,
    pub sig_b64: String,
}

pub struct PreparedTransportSigner {
    raw: *mut nxms_ms_signer_ctx,
}

impl PreparedTransportSigner {
    pub fn new(sender_sig_sk: &[u8]) -> Result<Self> {
        if sender_sig_sk.is_empty() || sender_sig_sk.len() > NXMS_MAX_SIG_SK_LEN {
            return Err(anyhow!(
                "invalid sender sig sk length {} (expected 1..={})",
                sender_sig_sk.len(),
                NXMS_MAX_SIG_SK_LEN
            ));
        }
        let raw = unsafe { nxms_ms_signer_ctx_new(sender_sig_sk.as_ptr(), sender_sig_sk.len()) };
        if raw.is_null() {
            return Err(anyhow!("nxms_ms_signer_ctx_new failed"));
        }
        Ok(Self { raw })
    }
}

impl Drop for PreparedTransportSigner {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { nxms_ms_signer_ctx_free(self.raw) };
            self.raw = ptr::null_mut();
        }
    }
}

type EncryptOutputs = (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>);

#[allow(clippy::too_many_arguments)]
pub fn encrypt_for_context(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    context_id: &[u8; NXMS_CONTEXT_ID_LEN],
    seq: u64,
    recipient_kem_pk: &[u8],
    sender_sig_sk: &[u8],
    plaintext: &[u8],
) -> Result<SealedPacket> {
    encrypt(
        sender_id,
        to_id,
        msg_type,
        context_id,
        seq,
        recipient_kem_pk,
        sender_sig_sk,
        plaintext,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn encrypt_for_context_with_signer(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    context_id: &[u8; NXMS_CONTEXT_ID_LEN],
    seq: u64,
    recipient_kem_pk: &[u8],
    signer: &PreparedTransportSigner,
    plaintext: &[u8],
) -> Result<SealedPacket> {
    encrypt_internal(
        sender_id,
        to_id,
        msg_type,
        context_id,
        seq,
        recipient_kem_pk,
        None,
        Some(signer),
        plaintext,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn decrypt_for_context(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    context_id: &[u8; NXMS_CONTEXT_ID_LEN],
    seq: u64,
    sealed: &SealedPacket,
    recipient_kem_sk: &[u8],
    sender_sig_pk: &[u8],
) -> Result<Vec<u8>> {
    decrypt(
        sender_id,
        to_id,
        msg_type,
        context_id,
        seq,
        sealed,
        recipient_kem_sk,
        sender_sig_pk,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn encrypt(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    escrow_id: &[u8; 16],
    seq: u64,
    recipient_kem_pk: &[u8],
    sender_sig_sk: &[u8],
    plaintext: &[u8],
) -> Result<SealedPacket> {
    encrypt_internal(
        sender_id,
        to_id,
        msg_type,
        escrow_id,
        seq,
        recipient_kem_pk,
        Some(sender_sig_sk),
        None,
        plaintext,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn encrypt_with_signer(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    escrow_id: &[u8; 16],
    seq: u64,
    recipient_kem_pk: &[u8],
    signer: &PreparedTransportSigner,
    plaintext: &[u8],
) -> Result<SealedPacket> {
    encrypt_internal(
        sender_id,
        to_id,
        msg_type,
        escrow_id,
        seq,
        recipient_kem_pk,
        None,
        Some(signer),
        plaintext,
    )
}

#[allow(clippy::too_many_arguments)]
fn encrypt_internal(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    escrow_id: &[u8; 16],
    seq: u64,
    recipient_kem_pk: &[u8],
    sender_sig_sk: Option<&[u8]>,
    signer: Option<&PreparedTransportSigner>,
    plaintext: &[u8],
) -> Result<SealedPacket> {
    if plaintext.len() > NXMS_MAX_PAYLOAD {
        return Err(anyhow!("payload too large"));
    }
    if seq == 0 {
        return Err(anyhow!("seq must be > 0"));
    }
    if recipient_kem_pk.len() < NXMS_MIN_KEM_BYTES || recipient_kem_pk.len() > NXMS_MAX_KEM_PK_LEN {
        return Err(anyhow!(
            "invalid recipient kem pk length {} (expected {}..={})",
            recipient_kem_pk.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_PK_LEN
        ));
    }
    if signer.is_none() {
        let sender_sig_sk = sender_sig_sk.ok_or_else(|| anyhow!("missing sender sig sk"))?;
        if sender_sig_sk.is_empty() || sender_sig_sk.len() > NXMS_MAX_SIG_SK_LEN {
            return Err(anyhow!(
                "invalid sender sig sk length {} (expected 1..={})",
                sender_sig_sk.len(),
                NXMS_MAX_SIG_SK_LEN
            ));
        }
    }

    let sender_id = CString::new(sender_id)?;
    let to_id = CString::new(to_id)?;
    let msg_type = CString::new(msg_type)?;

    let mut kem_ct_ptr: *mut u8 = ptr::null_mut();
    let mut kem_ct_len: usize = 0;
    let mut nonce_ptr: *mut u8 = ptr::null_mut();
    let mut nonce_len: usize = 0;
    let mut ciphertext_ptr: *mut u8 = ptr::null_mut();
    let mut ciphertext_len: usize = 0;
    let mut tag_ptr: *mut u8 = ptr::null_mut();
    let mut tag_len: usize = 0;
    let mut sig_ptr: *mut u8 = ptr::null_mut();
    let mut sig_len: usize = 0;

    let rc = unsafe {
        match signer {
            Some(signer) => nxms_ms_encrypt_packet_with_signer(
                sender_id.as_ptr(),
                to_id.as_ptr(),
                msg_type.as_ptr(),
                escrow_id.as_ptr(),
                seq,
                recipient_kem_pk.as_ptr(),
                recipient_kem_pk.len(),
                signer.raw,
                plaintext.as_ptr(),
                plaintext.len(),
                &mut kem_ct_ptr,
                &mut kem_ct_len,
                &mut nonce_ptr,
                &mut nonce_len,
                &mut ciphertext_ptr,
                &mut ciphertext_len,
                &mut tag_ptr,
                &mut tag_len,
                &mut sig_ptr,
                &mut sig_len,
            ),
            None => {
                let sender_sig_sk =
                    sender_sig_sk.expect("sender_sig_sk validated when signer is absent");
                nxms_ms_encrypt_packet(
                    sender_id.as_ptr(),
                    to_id.as_ptr(),
                    msg_type.as_ptr(),
                    escrow_id.as_ptr(),
                    seq,
                    recipient_kem_pk.as_ptr(),
                    recipient_kem_pk.len(),
                    sender_sig_sk.as_ptr(),
                    sender_sig_sk.len(),
                    plaintext.as_ptr(),
                    plaintext.len(),
                    &mut kem_ct_ptr,
                    &mut kem_ct_len,
                    &mut nonce_ptr,
                    &mut nonce_len,
                    &mut ciphertext_ptr,
                    &mut ciphertext_len,
                    &mut tag_ptr,
                    &mut tag_len,
                    &mut sig_ptr,
                    &mut sig_len,
                )
            }
        }
    };
    if rc != 0 {
        // C contract resets output pointers to NULL on error.
        return Err(anyhow!("nxms_ms_encrypt_packet failed rc={}", rc));
    }

    let (kem_ct, nonce, ciphertext, tag, sig) = unsafe {
        take_encrypt_outputs(
            kem_ct_ptr,
            kem_ct_len,
            nonce_ptr,
            nonce_len,
            ciphertext_ptr,
            ciphertext_len,
            tag_ptr,
            tag_len,
            sig_ptr,
            sig_len,
        )
    }?;

    if nonce.len() != NXMS_NONCE_LEN {
        return Err(anyhow!("invalid nonce length {}", nonce.len()));
    }
    if tag.len() != NXMS_TAG_LEN {
        return Err(anyhow!("invalid tag length {}", tag.len()));
    }
    if sig.is_empty() {
        return Err(anyhow!("invalid signature length 0"));
    }

    Ok(SealedPacket {
        kem_ct_b64: B64.encode(kem_ct),
        nonce_b64: B64.encode(nonce),
        ciphertext_b64: B64.encode(ciphertext),
        tag_b64: B64.encode(tag),
        sig_b64: B64.encode(sig),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn decrypt(
    sender_id: &str,
    to_id: &str,
    msg_type: &str,
    escrow_id: &[u8; 16],
    seq: u64,
    sealed: &SealedPacket,
    recipient_kem_sk: &[u8],
    sender_sig_pk: &[u8],
) -> Result<Vec<u8>> {
    if seq == 0 {
        return Err(anyhow!("seq must be > 0"));
    }
    if recipient_kem_sk.len() < NXMS_MIN_KEM_BYTES || recipient_kem_sk.len() > NXMS_MAX_KEM_SK_LEN {
        return Err(anyhow!(
            "invalid recipient kem sk length {} (expected {}..={})",
            recipient_kem_sk.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_SK_LEN
        ));
    }
    if sender_sig_pk.is_empty() || sender_sig_pk.len() > NXMS_MAX_SIG_PK_LEN {
        return Err(anyhow!(
            "invalid sender sig pk length {} (expected 1..={})",
            sender_sig_pk.len(),
            NXMS_MAX_SIG_PK_LEN
        ));
    }

    let sender_id = CString::new(sender_id)?;
    let to_id = CString::new(to_id)?;
    let msg_type = CString::new(msg_type)?;

    validate_b64_input_len(&sealed.kem_ct_b64, "kem_ct", NXMS_MAX_KEM_CT_LEN)?;
    validate_b64_input_len(&sealed.nonce_b64, "nonce", NXMS_NONCE_LEN)?;
    validate_b64_input_len(&sealed.ciphertext_b64, "ciphertext", NXMS_MAX_PAYLOAD)?;
    validate_b64_input_len(&sealed.tag_b64, "tag", NXMS_TAG_LEN)?;
    validate_b64_input_len(&sealed.sig_b64, "signature", FF_FALCON_SIG_MAX)?;

    let kem_ct = B64.decode(&sealed.kem_ct_b64)?;
    let nonce = B64.decode(&sealed.nonce_b64)?;
    let ciphertext = B64.decode(&sealed.ciphertext_b64)?;
    let tag = B64.decode(&sealed.tag_b64)?;
    let sig = B64.decode(&sealed.sig_b64)?;

    if kem_ct.len() < NXMS_MIN_KEM_BYTES || kem_ct.len() > NXMS_MAX_KEM_CT_LEN {
        return Err(anyhow!(
            "invalid kem_ct length {} (expected {}..={})",
            kem_ct.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_CT_LEN
        ));
    }
    if nonce.len() != NXMS_NONCE_LEN {
        return Err(anyhow!(
            "invalid nonce length {} (expected {})",
            nonce.len(),
            NXMS_NONCE_LEN
        ));
    }
    if tag.len() != NXMS_TAG_LEN {
        return Err(anyhow!(
            "invalid tag length {} (expected {})",
            tag.len(),
            NXMS_TAG_LEN
        ));
    }
    if ciphertext.len() > NXMS_MAX_PAYLOAD {
        return Err(anyhow!(
            "invalid ciphertext length {} (max {})",
            ciphertext.len(),
            NXMS_MAX_PAYLOAD
        ));
    }
    if sig.is_empty() || sig.len() > FF_FALCON_SIG_MAX {
        return Err(anyhow!(
            "invalid signature length {} (expected 1..={})",
            sig.len(),
            FF_FALCON_SIG_MAX
        ));
    }

    let mut out_ptr: *mut u8 = ptr::null_mut();
    let mut out_len: usize = 0;
    let rc = unsafe {
        nxms_ms_verify_decrypt(
            sender_id.as_ptr(),
            to_id.as_ptr(),
            msg_type.as_ptr(),
            escrow_id.as_ptr(),
            seq,
            kem_ct.as_ptr(),
            kem_ct.len(),
            nonce.as_ptr(),
            nonce.len(),
            ciphertext.as_ptr(),
            ciphertext.len(),
            tag.as_ptr(),
            tag.len(),
            sig.as_ptr(),
            sig.len(),
            recipient_kem_sk.as_ptr(),
            recipient_kem_sk.len(),
            sender_sig_pk.as_ptr(),
            sender_sig_pk.len(),
            &mut out_ptr,
            &mut out_len,
        )
    };
    if rc != 0 {
        // C contract resets output pointers to NULL on error.
        return Err(anyhow!("nxms_ms_verify_decrypt failed rc={}", rc));
    }

    unsafe { take_alloc_checked_secure(out_ptr, out_len, NXMS_MAX_PAYLOAD, "plaintext") }
}

pub fn suite_kem_id() -> &'static str {
    NXMS_KEM_ID
}
pub fn suite_sig_id() -> &'static str {
    NXMS_SIG_ID
}

pub fn falcon_keygen() -> Result<(Zeroizing<Vec<u8>>, Vec<u8>)> {
    falcon_keygen_internal(None)
}

pub fn falcon_keygen_seeded(
    seed: &[u8; FF_FALCON_TEST_SEED_LEN],
) -> Result<(Zeroizing<Vec<u8>>, Vec<u8>)> {
    falcon_keygen_internal(Some(seed))
}

fn falcon_keygen_internal(
    seed: Option<&[u8; FF_FALCON_TEST_SEED_LEN]>,
) -> Result<(Zeroizing<Vec<u8>>, Vec<u8>)> {
    let mut sig_sk = vec![0u8; NXMS_MAX_SIG_SK_LEN];
    let mut sig_pk = vec![0u8; NXMS_MAX_SIG_PK_LEN];
    let mut sig_sk_len: usize = sig_sk.len();
    let mut sig_pk_len: usize = sig_pk.len();
    let rc = unsafe {
        match seed {
            Some(seed) => ff_falcon_keygen_seeded(
                seed.as_ptr(),
                seed.len(),
                sig_sk.as_mut_ptr(),
                &mut sig_sk_len,
                sig_pk.as_mut_ptr(),
                &mut sig_pk_len,
            ),
            None => ff_falcon_keygen(
                sig_sk.as_mut_ptr(),
                &mut sig_sk_len,
                sig_pk.as_mut_ptr(),
                &mut sig_pk_len,
            ),
        }
    };
    if rc != 0 {
        return Err(anyhow!("ff_falcon_keygen failed rc={}", rc));
    }
    sig_sk.truncate(sig_sk_len);
    sig_pk.truncate(sig_pk_len);
    Ok((Zeroizing::new(sig_sk), sig_pk))
}

pub fn falcon_sign_ct(sig_sk: &[u8], msg: &[u8]) -> Result<Vec<u8>> {
    falcon_sign_ct_internal(None, sig_sk, msg)
}

pub fn falcon_sign_ct_seeded(
    seed: &[u8; FF_FALCON_TEST_SEED_LEN],
    sig_sk: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>> {
    falcon_sign_ct_internal(Some(seed), sig_sk, msg)
}

fn falcon_sign_ct_internal(
    seed: Option<&[u8; FF_FALCON_TEST_SEED_LEN]>,
    sig_sk: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>> {
    if sig_sk.is_empty() || sig_sk.len() > NXMS_MAX_SIG_SK_LEN {
        return Err(anyhow!("invalid Falcon secret key length"));
    }
    let mut sig = vec![0u8; FF_FALCON_SIG_MAX];
    let mut sig_len = sig.len();
    let rc = unsafe {
        match seed {
            Some(seed) => ff_falcon_sign_ct_seeded(
                seed.as_ptr(),
                seed.len(),
                sig_sk.as_ptr(),
                sig_sk.len(),
                msg.as_ptr(),
                msg.len(),
                sig.as_mut_ptr(),
                &mut sig_len,
            ),
            None => ff_falcon_sign_ct(
                sig_sk.as_ptr(),
                sig_sk.len(),
                msg.as_ptr(),
                msg.len(),
                sig.as_mut_ptr(),
                &mut sig_len,
            ),
        }
    };
    if rc != 0 {
        return Err(anyhow!("ff_falcon_sign_ct failed rc={}", rc));
    }
    sig.truncate(sig_len);
    Ok(sig)
}

pub fn falcon_sign_ct_prepared(sig_sk: &[u8], msg: &[u8]) -> Result<Vec<u8>> {
    falcon_sign_ct_prepared_internal(None, sig_sk, msg)
}

pub fn falcon_sign_ct_prepared_seeded(
    seed: &[u8; FF_FALCON_TEST_SEED_LEN],
    sig_sk: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>> {
    falcon_sign_ct_prepared_internal(Some(seed), sig_sk, msg)
}

fn falcon_sign_ct_prepared_internal(
    seed: Option<&[u8; FF_FALCON_TEST_SEED_LEN]>,
    sig_sk: &[u8],
    msg: &[u8],
) -> Result<Vec<u8>> {
    if sig_sk.is_empty() || sig_sk.len() > NXMS_MAX_SIG_SK_LEN {
        return Err(anyhow!("invalid Falcon secret key length"));
    }

    let mut prepared = MaybeUninit::<ff_falcon_prepared_sk_t>::zeroed();
    let rc = unsafe { ff_falcon_prepare_sk(sig_sk.as_ptr(), sig_sk.len(), prepared.as_mut_ptr()) };
    if rc != 0 {
        return Err(anyhow!("ff_falcon_prepare_sk failed rc={}", rc));
    }

    let mut prepared = unsafe { prepared.assume_init() };
    let mut sig = vec![0u8; FF_FALCON_SIG_MAX];
    let mut sig_len = sig.len();
    let rc = unsafe {
        match seed {
            Some(seed) => ff_falcon_sign_ct_prepared_seeded(
                seed.as_ptr(),
                seed.len(),
                &prepared,
                msg.as_ptr(),
                msg.len(),
                sig.as_mut_ptr(),
                &mut sig_len,
            ),
            None => ff_falcon_sign_ct_prepared(
                &prepared,
                msg.as_ptr(),
                msg.len(),
                sig.as_mut_ptr(),
                &mut sig_len,
            ),
        }
    };
    unsafe { ff_falcon_clear_prepared_sk(&mut prepared) };
    if rc != 0 {
        return Err(anyhow!("ff_falcon_sign_ct_prepared failed rc={}", rc));
    }
    sig.truncate(sig_len);
    Ok(sig)
}

pub fn falcon_verify(sig_pk: &[u8], msg: &[u8], sig: &[u8]) -> Result<()> {
    if sig_pk.is_empty() || sig_pk.len() > NXMS_MAX_SIG_PK_LEN {
        return Err(anyhow!("invalid Falcon public key length"));
    }
    if sig.is_empty() || sig.len() > FF_FALCON_SIG_MAX {
        return Err(anyhow!("invalid Falcon signature length"));
    }
    let rc = unsafe {
        ff_falcon_verify(
            sig_pk.as_ptr(),
            sig_pk.len(),
            msg.as_ptr(),
            msg.len(),
            sig.as_ptr(),
            sig.len(),
        )
    };
    if rc != 0 {
        return Err(anyhow!("ff_falcon_verify failed rc={}", rc));
    }
    Ok(())
}

pub fn kem_encaps(recipient_kem_pk: &[u8]) -> Result<(Vec<u8>, Zeroizing<Vec<u8>>)> {
    if recipient_kem_pk.len() < NXMS_MIN_KEM_BYTES || recipient_kem_pk.len() > NXMS_MAX_KEM_PK_LEN {
        return Err(anyhow!(
            "invalid recipient kem pk length {} (expected {}..={})",
            recipient_kem_pk.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_PK_LEN
        ));
    }

    let alg = CString::new(NXMS_KEM_ID)?;
    let mut ct_ptr: *mut u8 = ptr::null_mut();
    let mut ct_len: usize = 0;
    let mut ss_ptr: *mut u8 = ptr::null_mut();
    let mut ss_len: usize = 0;

    let rc = unsafe {
        ff_kem_encaps(
            alg.as_ptr(),
            recipient_kem_pk.as_ptr(),
            recipient_kem_pk.len(),
            &mut ct_ptr,
            &mut ct_len,
            &mut ss_ptr,
            &mut ss_len,
        )
    };
    if rc != 0 {
        return Err(anyhow!("ff_kem_encaps failed rc={}", rc));
    }

    let ct = unsafe { take_alloc_checked(ct_ptr, ct_len, NXMS_MAX_KEM_CT_LEN, "kem_ct") }?;
    let ss =
        unsafe { take_alloc_checked_secure(ss_ptr, ss_len, NXMS_MAX_KEM_CT_LEN, "shared_secret") }?;
    Ok((ct, Zeroizing::new(ss)))
}

pub fn kem_decaps(recipient_kem_sk: &[u8], kem_ct: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    if recipient_kem_sk.len() < NXMS_MIN_KEM_BYTES || recipient_kem_sk.len() > NXMS_MAX_KEM_SK_LEN {
        return Err(anyhow!(
            "invalid recipient kem sk length {} (expected {}..={})",
            recipient_kem_sk.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_SK_LEN
        ));
    }
    if kem_ct.len() < NXMS_MIN_KEM_BYTES || kem_ct.len() > NXMS_MAX_KEM_CT_LEN {
        return Err(anyhow!(
            "invalid kem ct length {} (expected {}..={})",
            kem_ct.len(),
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_CT_LEN
        ));
    }

    let alg = CString::new(NXMS_KEM_ID)?;
    let mut ss_ptr: *mut u8 = ptr::null_mut();
    let mut ss_len: usize = 0;
    let rc = unsafe {
        ff_kem_decaps(
            alg.as_ptr(),
            recipient_kem_sk.as_ptr(),
            recipient_kem_sk.len(),
            kem_ct.as_ptr(),
            kem_ct.len(),
            &mut ss_ptr,
            &mut ss_len,
        )
    };
    if rc != 0 {
        return Err(anyhow!("ff_kem_decaps failed rc={}", rc));
    }

    Ok(Zeroizing::new(unsafe {
        take_alloc_checked_secure(ss_ptr, ss_len, NXMS_MAX_KEM_CT_LEN, "shared_secret")
    }?))
}

pub fn random_xchacha20poly1305_nonce() -> [u8; XCHACHA20POLY1305_NONCE_LEN] {
    ensure_sodium_initialized().expect("libsodium init");
    let mut nonce = [0u8; XCHACHA20POLY1305_NONCE_LEN];
    unsafe { randombytes_buf(nonce.as_mut_ptr() as *mut c_void, nonce.len()) };
    nonce
}

pub fn xchacha20poly1305_encrypt(
    key: &[u8],
    nonce: &[u8; XCHACHA20POLY1305_NONCE_LEN],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<(Vec<u8>, [u8; XCHACHA20POLY1305_TAG_LEN])> {
    if key.len() != XCHACHA20POLY1305_KEY_LEN {
        return Err(anyhow!(
            "invalid xchacha20-poly1305 key length {} (expected {})",
            key.len(),
            XCHACHA20POLY1305_KEY_LEN
        ));
    }
    ensure_sodium_initialized()?;

    let mut combined = vec![0u8; plaintext.len() + XCHACHA20POLY1305_TAG_LEN];
    let mut combined_len = 0u64;
    let rc = unsafe {
        crypto_aead_xchacha20poly1305_ietf_encrypt(
            combined.as_mut_ptr(),
            &mut combined_len,
            plaintext.as_ptr(),
            plaintext.len() as u64,
            aad.as_ptr(),
            aad.len() as u64,
            ptr::null(),
            nonce.as_ptr(),
            key.as_ptr(),
        )
    };
    if rc != 0 {
        return Err(anyhow!("crypto_aead_xchacha20poly1305_ietf_encrypt failed"));
    }
    combined.truncate(combined_len as usize);
    if combined.len() < XCHACHA20POLY1305_TAG_LEN {
        return Err(anyhow!("combined ciphertext shorter than tag"));
    }

    let split = combined.len() - XCHACHA20POLY1305_TAG_LEN;
    let mut tag = [0u8; XCHACHA20POLY1305_TAG_LEN];
    tag.copy_from_slice(&combined[split..]);
    combined.truncate(split);
    Ok((combined, tag))
}

pub fn xchacha20poly1305_decrypt(
    key: &[u8],
    nonce: &[u8; XCHACHA20POLY1305_NONCE_LEN],
    ciphertext: &[u8],
    tag: &[u8; XCHACHA20POLY1305_TAG_LEN],
    aad: &[u8],
) -> Result<Vec<u8>> {
    if key.len() != XCHACHA20POLY1305_KEY_LEN {
        return Err(anyhow!(
            "invalid xchacha20-poly1305 key length {} (expected {})",
            key.len(),
            XCHACHA20POLY1305_KEY_LEN
        ));
    }
    ensure_sodium_initialized()?;

    let mut combined = Vec::with_capacity(ciphertext.len() + XCHACHA20POLY1305_TAG_LEN);
    combined.extend_from_slice(ciphertext);
    combined.extend_from_slice(tag);

    let mut plaintext = vec![0u8; ciphertext.len()];
    let mut plaintext_len = 0u64;
    let rc = unsafe {
        crypto_aead_xchacha20poly1305_ietf_decrypt(
            plaintext.as_mut_ptr(),
            &mut plaintext_len,
            ptr::null_mut(),
            combined.as_ptr(),
            combined.len() as u64,
            aad.as_ptr(),
            aad.len() as u64,
            nonce.as_ptr(),
            key.as_ptr(),
        )
    };
    if rc != 0 {
        return Err(anyhow!("crypto_aead_xchacha20poly1305_ietf_decrypt failed"));
    }
    plaintext.truncate(plaintext_len as usize);
    Ok(plaintext)
}

unsafe fn free_if_not_null(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe { nxms_ms_free(ptr as *mut c_void) };
    }
}

unsafe fn free_if_not_null_secure(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe { nxms_ms_free_secure(ptr as *mut c_void, len) };
    }
}

fn decode_key_b64(value: &str, label: &str, min_len: usize, max_len: usize) -> Result<Vec<u8>> {
    validate_b64_input_len(value, label, max_len)?;
    let out = B64.decode(value.as_bytes())?;
    if out.len() < min_len || out.len() > max_len {
        return Err(anyhow!(
            "invalid decoded {label} length {} (expected {min_len}..={max_len})",
            out.len()
        ));
    }
    Ok(out)
}

fn validate_b64_input_len(value: &str, label: &str, max_decoded_len: usize) -> Result<()> {
    let max_b64_len = max_b64_len_for_decoded(max_decoded_len)
        .ok_or_else(|| anyhow!("base64 length cap overflow for {label}"))?;
    if value.len() > max_b64_len {
        return Err(anyhow!(
            "{label} base64 too long: {} > {}",
            value.len(),
            max_b64_len
        ));
    }
    Ok(())
}

fn max_b64_len_for_decoded(max_decoded_len: usize) -> Option<usize> {
    let quads = max_decoded_len.checked_add(2)? / 3;
    quads.checked_mul(4)
}

fn ensure_sodium_initialized() -> Result<()> {
    let rc = unsafe { sodium_init() };
    if rc < 0 {
        return Err(anyhow!("libsodium initialization failed"));
    }
    Ok(())
}

unsafe fn copy_kem_key_material(k: &ff_kem_keys_t) -> Result<(Vec<u8>, Vec<u8>)> {
    let pk = unsafe {
        copy_raw_borrowed_checked(
            k.pk,
            k.pk_len,
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_PK_LEN,
            "kem pk",
        )
    }?;
    let sk = unsafe {
        copy_raw_borrowed_checked(
            k.sk,
            k.sk_len,
            NXMS_MIN_KEM_BYTES,
            NXMS_MAX_KEM_SK_LEN,
            "kem sk",
        )
    }?;
    Ok((pk, sk))
}

unsafe fn copy_raw_borrowed_checked(
    ptr: *const u8,
    len: usize,
    min_len: usize,
    max_len: usize,
    label: &str,
) -> Result<Vec<u8>> {
    if len < min_len {
        return Err(anyhow!("invalid {label} length {len}"));
    }
    if len > max_len {
        return Err(anyhow!("invalid {label} length {len}"));
    }
    if ptr.is_null() {
        return Err(anyhow!("{label} pointer is null"));
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec())
}

unsafe fn take_alloc_checked(
    ptr: *mut u8,
    len: usize,
    max_len: usize,
    label: &str,
) -> Result<Vec<u8>> {
    if ptr.is_null() {
        if len == 0 {
            return Ok(Vec::new());
        }
        return Err(anyhow!("{label} pointer is null with non-zero length"));
    }
    if len > max_len {
        unsafe { free_if_not_null(ptr) };
        return Err(anyhow!("{label} length exceeds max: {len} > {max_len}"));
    }
    if len == 0 {
        unsafe { free_if_not_null(ptr) };
        return Ok(Vec::new());
    }
    let out = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    unsafe { free_if_not_null(ptr) };
    Ok(out)
}

unsafe fn take_alloc_checked_secure(
    ptr: *mut u8,
    len: usize,
    max_len: usize,
    label: &str,
) -> Result<Vec<u8>> {
    if ptr.is_null() {
        if len == 0 {
            return Ok(Vec::new());
        }
        return Err(anyhow!("{label} pointer is null with non-zero length"));
    }
    if len > max_len {
        unsafe { free_if_not_null_secure(ptr, len) };
        return Err(anyhow!("{label} length exceeds max: {len} > {max_len}"));
    }
    if len == 0 {
        unsafe { free_if_not_null_secure(ptr, len) };
        return Ok(Vec::new());
    }
    let out = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    unsafe { free_if_not_null_secure(ptr, len) };
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
unsafe fn take_encrypt_outputs(
    kem_ct_ptr: *mut u8,
    kem_ct_len: usize,
    nonce_ptr: *mut u8,
    nonce_len: usize,
    ciphertext_ptr: *mut u8,
    ciphertext_len: usize,
    tag_ptr: *mut u8,
    tag_len: usize,
    sig_ptr: *mut u8,
    sig_len: usize,
) -> Result<EncryptOutputs> {
    let kem_ct = match unsafe {
        take_alloc_checked(kem_ct_ptr, kem_ct_len, NXMS_MAX_KEM_CT_LEN, "kem_ct")
    } {
        Ok(v) => v,
        Err(err) => {
            unsafe {
                free_if_not_null(nonce_ptr);
                free_if_not_null(ciphertext_ptr);
                free_if_not_null(tag_ptr);
                free_if_not_null(sig_ptr);
            }
            return Err(err);
        }
    };
    let nonce = match unsafe { take_alloc_checked(nonce_ptr, nonce_len, NXMS_NONCE_LEN, "nonce") } {
        Ok(v) => v,
        Err(err) => {
            unsafe {
                free_if_not_null(ciphertext_ptr);
                free_if_not_null(tag_ptr);
                free_if_not_null(sig_ptr);
            }
            return Err(err);
        }
    };
    let ciphertext = match unsafe {
        take_alloc_checked(
            ciphertext_ptr,
            ciphertext_len,
            NXMS_MAX_PAYLOAD,
            "ciphertext",
        )
    } {
        Ok(v) => v,
        Err(err) => {
            unsafe {
                free_if_not_null(tag_ptr);
                free_if_not_null(sig_ptr);
            }
            return Err(err);
        }
    };
    let tag = match unsafe { take_alloc_checked(tag_ptr, tag_len, NXMS_TAG_LEN, "tag") } {
        Ok(v) => v,
        Err(err) => {
            unsafe { free_if_not_null(sig_ptr) };
            return Err(err);
        }
    };
    let sig = unsafe { take_alloc_checked(sig_ptr, sig_len, FF_FALCON_SIG_MAX, "signature") }?;
    Ok((kem_ct, nonce, ciphertext, tag, sig))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn falcon_sign_and_verify_roundtrip() {
        let keys = Keys::generate().expect("keygen");
        let sig_sk = keys.sig_sk_zeroizing().expect("sig sk");
        let sig_pk = keys.sig_pk().expect("sig pk");
        let msg = b"snapshot-hash-123";

        let sig = falcon_sign_ct(sig_sk.as_slice(), msg).expect("sign");
        falcon_verify(&sig_pk, msg, &sig).expect("verify");
    }

    #[cfg(unix)]
    #[test]
    fn write_json_sets_private_file_mode() {
        let p = std::env::temp_dir().join(format!(
            "nxms_keys_write_json_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        let keys = Keys {
            kem_sk_b64: SecretB64::new("a".to_string()),
            kem_pk_b64: "b".to_string(),
            sig_sk_b64: SecretB64::new("c".to_string()),
            sig_pk_b64: "d".to_string(),
        };
        keys.write_json(p.to_str().expect("path"))
            .expect("write json");

        let mode = std::fs::metadata(&p)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        let _ = std::fs::remove_file(&p);
        assert_eq!(mode, 0o600);
    }
}
