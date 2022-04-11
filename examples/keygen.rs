use rsa::{pkcs1, RsaPrivateKey, RsaPublicKey};
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
fn main()
{
    let mut rng = rand::thread_rng();
    let bits = 512;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    let mut options = OpenOptions::new();
    let options = options.mode(0o600).write(true).create_new(true);

    fs::remove_file("private.pem").ok();
    fs::remove_file("public.pem").ok();

    options.open("private.pem").unwrap();
    options.open("public.pem").unwrap();

    fs::write(
        "private.pem",
        pkcs1::EncodeRsaPrivateKey::to_pkcs1_pem(&priv_key, rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_str(),
    )
    .ok();
    fs::write(
        "public.pem",
        pkcs1::EncodeRsaPublicKey::to_pkcs1_pem(&pub_key, rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_str(),
    )
    .ok();
}
