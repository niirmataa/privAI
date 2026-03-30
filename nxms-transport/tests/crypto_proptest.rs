#![cfg(feature = "crypto")]

use nxms_transport::crypto::{Keys, decrypt, encrypt};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 8,
        .. ProptestConfig::default()
    })]

    #[test]
    fn roundtrip_arbitrary_plaintext_and_seq(
        plaintext in proptest::collection::vec(any::<u8>(), 0..=4096),
        seq in 1u64..=u64::MAX,
    ) {
        let sender = Keys::generate().expect("sender keys");
        let recipient = Keys::generate().expect("recipient keys");

        let sender_sig_sk = sender.sig_sk_zeroizing().expect("sender sig sk");
        let sender_sig_pk = sender.sig_pk().expect("sender sig pk");
        let recipient_kem_pk = recipient.kem_pk().expect("recipient kem pk");
        let recipient_kem_sk = recipient.kem_sk_zeroizing().expect("recipient kem sk");
        let escrow_id = [0xAAu8; 16];

        let sealed = encrypt(
            "alice",
            "bob",
            "tx_sign_req",
            &escrow_id,
            seq,
            &recipient_kem_pk,
            sender_sig_sk.as_slice(),
            &plaintext,
        )
        .expect("encrypt");

        let out = decrypt(
            "alice",
            "bob",
            "tx_sign_req",
            &escrow_id,
            seq,
            &sealed,
            recipient_kem_sk.as_slice(),
            &sender_sig_pk,
        )
        .expect("decrypt");

        prop_assert_eq!(out, plaintext);
    }
}
