#[allow(unused)]
use k256::{
    ecdsa::{signature::Signer, Signature, SigningKey},
    SecretKey,
};
use std::fs;

// Verification
use k256::ecdsa::{signature::Verifier, VerifyingKey};
#[allow(unused)]
use rand::thread_rng;

fn main()
{
    {
        // Signing
        let signing_key = SigningKey::random(thread_rng());
        let sign_bytes = signing_key.to_bytes();
        fs::write("key.sign", sign_bytes).unwrap();

        let verify_key = VerifyingKey::from(&signing_key);
        let verify_bytes = verify_key.to_bytes();
        fs::write("key.verify", verify_bytes).unwrap();
    }
    /*
    {
        // Signing
        let signing_key = SigningKey::random(thread_rng());
        let sign_bytes = signing_key.to_bytes();
        fs::write("key.sign3", sign_bytes).unwrap();

        let verify_key = VerifyingKey::from(&signing_key);
        let verify_bytes = verify_key.to_bytes();
        fs::write("key.verify3", verify_bytes).unwrap();
    }
    */

    let signature: Signature;

    {
        let message = b"critical message";
        let signing_key = SigningKey::from_bytes(&fs::read("key.sign").unwrap()).unwrap();

        // Note: the signature type must be annotated or otherwise inferrable as
        // `Signer` has many impls of the `Signer` trait (for both regular and
        // recoverable signature types).
        signature = signing_key.sign(message);
    }

    {
        let verify_key = VerifyingKey::from_sec1_bytes(&fs::read("key.verify").unwrap()).unwrap();
        // Serialize with `::to_encoded_point()`
        verify_key.verify(b"critical message", &signature).unwrap();
    }
}
