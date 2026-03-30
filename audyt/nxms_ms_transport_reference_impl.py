from __future__ import annotations

import json
import sys
from hashlib import shake_256
from pathlib import Path


AAD_PREFIX = b"NXMS-AAD-v1"
SIG_PREFIX = b"NXMS-SIG-v1"
CTHASH_PREFIX = b"NXMS-CTHASH-v1"
KDF_PREFIX = b"NXMS-KDF-v1"
TAG_PREFIX = b"NXMS-TAG-v1"
STREAM_PREFIX = b"NXMS-STREAM-v1"
NXMS_KEM_ID = b"FrodoKEM-640-SHAKE"
NXMS_SIG_ID = b"Falcon-1024-CT"
KEM_BINDING_HASH32 = "hash32"
KEM_BINDING_FULL = "full"


def u32be(value: int) -> bytes:
    return value.to_bytes(4, "big")


def u64be(value: int) -> bytes:
    return value.to_bytes(8, "big")


def shake(parts: list[bytes], out_len: int) -> bytes:
    h = shake_256()
    for part in parts:
        h.update(part)
    return h.digest(out_len)


def derive_key(ss: bytes, escrow_id: bytes, label: bytes) -> bytes:
    return shake([KDF_PREFIX, u32be(len(ss)), ss, escrow_id, label], 32)


def ct_hash(kem_ct: bytes) -> bytes:
    return shake([CTHASH_PREFIX, kem_ct], 32)


def kem_binding_bytes(kem_ct: bytes, mode: str) -> bytes:
    if mode == KEM_BINDING_HASH32:
        return ct_hash(kem_ct)
    if mode == KEM_BINDING_FULL:
        return u32be(len(kem_ct)) + kem_ct
    raise ValueError(f"unsupported kem binding mode: {mode}")


def build_aad(
    sender_id: str,
    to_id: str,
    msg_type: str,
    escrow_id: bytes,
    seq: int,
    kem_ct: bytes,
    kem_binding_mode: str = KEM_BINDING_HASH32,
) -> bytes:
    sender = sender_id.encode()
    to = to_id.encode()
    msg = msg_type.encode()
    return b"".join(
        [
            AAD_PREFIX,
            u32be(len(sender)),
            sender,
            u32be(len(to)),
            to,
            u32be(len(NXMS_KEM_ID)),
            NXMS_KEM_ID,
            u32be(len(NXMS_SIG_ID)),
            NXMS_SIG_ID,
            u32be(len(msg)),
            msg,
            escrow_id,
            u64be(seq),
            kem_binding_bytes(kem_ct, kem_binding_mode),
        ]
    )


def xor_keystream(plaintext: bytes, ke: bytes, nonce: bytes) -> bytes:
    stream = shake([STREAM_PREFIX, ke, nonce], len(plaintext))
    return bytes(a ^ b for a, b in zip(plaintext, stream))


def compute_tag(km: bytes, aad: bytes, nonce: bytes, ciphertext: bytes) -> bytes:
    return shake(
        [
            TAG_PREFIX,
            km,
            u32be(len(aad)),
            aad,
            u32be(len(nonce)),
            nonce,
            u32be(len(ciphertext)),
            ciphertext,
        ],
        32,
    )


def build_sig_message(aad: bytes, nonce: bytes, ciphertext: bytes, tag: bytes) -> bytes:
    return b"".join(
        [
            SIG_PREFIX,
            u32be(len(aad)),
            aad,
            u32be(len(nonce)),
            nonce,
            u32be(len(ciphertext)),
            ciphertext,
            u32be(len(tag)),
            tag,
        ]
    )


def load_vectors(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def verify_vectors(v: dict) -> None:
    sender_id = v["sender_id"]
    to_id = v["to_id"]
    msg_type = v["msg_type"]
    seq = v["seq"]
    kem_binding_mode = v.get("kem_binding_mode", KEM_BINDING_HASH32)
    ss = bytes.fromhex(v["ss_hex"])
    escrow_id = bytes.fromhex(v["escrow_id_hex"])
    kem_ct = bytes.fromhex(v["kem_ct_hex"])
    nonce = bytes.fromhex(v["nonce_hex"])
    plaintext = bytes.fromhex(v["plaintext_hex"])

    ke = derive_key(ss, escrow_id, b"ms-ke")
    km = derive_key(ss, escrow_id, b"ms-km")
    aad = build_aad(sender_id, to_id, msg_type, escrow_id, seq, kem_ct, kem_binding_mode)
    ciphertext = xor_keystream(plaintext, ke, nonce)
    tag = compute_tag(km, aad, nonce, ciphertext)
    sig_msg = build_sig_message(aad, nonce, ciphertext, tag)

    expected = {
        "ke_hex": ke.hex(),
        "km_hex": km.hex(),
        "ct_hash_hex": ct_hash(kem_ct).hex(),
        "aad_hex": aad.hex(),
        "ciphertext_hex": ciphertext.hex(),
        "tag_hex": tag.hex(),
        "sig_msg_hex": sig_msg.hex(),
    }

    mismatches = [name for name, got in expected.items() if got != v[name]]
    if mismatches:
        raise SystemExit(f"reference mismatch: {', '.join(mismatches)}")


def main() -> None:
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).with_name("nxms_ms_transport_reference_vectors_v1.json")
    verify_vectors(load_vectors(path))
    print("python reference ok")


if __name__ == "__main__":
    main()
