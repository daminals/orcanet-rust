use ring::{
    digest::{digest, SHA256},
    signature,
};

pub fn validate_payment(invoice: &str, wallet: &str, signature: &str, pubkey: &str) -> bool {
    // Verify the public key matches the wallet
    let pubkey = hex::decode(pubkey).unwrap();
    let sha256_digest = digest(&SHA256, pubkey.as_ref());
    let expected_wallet = hex::encode(sha256_digest.as_ref());
    if expected_wallet != wallet {
        return false;
    }

    // Verify the signature
    let pubkey = signature::UnparsedPublicKey::new(&signature::ED25519, pubkey);
    let signature = hex::decode(signature).unwrap();

    let expected = format!("{}{}", invoice, wallet);
    let expected = expected.as_bytes();
    match pubkey.verify(expected, &signature) {
        Ok(_) => return true,
        Err(_) => return false,
    }
}
