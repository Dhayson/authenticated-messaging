use rsa::{pkcs1, RsaPrivateKey, RsaPublicKey};
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
fn main()
{
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    let mut options = OpenOptions::new();
    let options = options.mode(0o600).write(true).create_new(true);

    fs::remove_file("private4.pem").ok();
    fs::remove_file("public4.pem").ok();

    options.open("private4.pem").unwrap();
    options.open("public4.pem").unwrap();

    fs::write(
        "private4.pem",
        pkcs1::EncodeRsaPrivateKey::to_pkcs1_pem(&priv_key, rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_str(),
    )
    .ok();
    fs::write(
        "public4.pem",
        pkcs1::EncodeRsaPublicKey::to_pkcs1_pem(&pub_key, rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .as_str(),
    )
    .ok();
}
