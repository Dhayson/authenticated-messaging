use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::{self, Rng};
use rsa::{errors::Error as RsaError, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

use k256::ecdsa::{SigningKey, VerifyingKey};

#[derive(Clone)]
pub enum RsaKey
{
    Private(RsaPrivateKey),
    Public(RsaPublicKey),
}

pub enum SignVerify
{
    Sign(SigningKey),
    Verify(VerifyingKey),
    MultiVerify(Vec<VerifyingKey>),
    Both(SigningKey, VerifyingKey),
}

impl RsaKey
{
    pub fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, RsaError>
    {
        let mut rng = rand::thread_rng();
        match self
        {
            RsaKey::Private(key) =>
            {
                key.encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
            }
            RsaKey::Public(key) =>
            {
                key.encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
            }
        }
    }

    pub fn decrypt(&self, cypher: &[u8]) -> Result<Vec<u8>, RsaError>
    {
        match self
        {
            RsaKey::Private(key) => key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), cypher),
            RsaKey::Public(_) => Err(RsaError::Decryption),
        }
    }
}

pub enum EncryptionErrors
{
    Rsa(RsaError),
    Aes(aes_gcm::Error),
}

impl From<RsaError> for EncryptionErrors
{
    fn from(err: RsaError) -> Self
    {
        Self::Rsa(err)
    }
}

impl From<aes_gcm::Error> for EncryptionErrors
{
    fn from(err: aes_gcm::Error) -> Self
    {
        Self::Aes(err)
    }
}
/// First Vec<u8> is the encrypted aes key
///
/// Second one is the encrypted text
pub fn aes_encrypt(rsa_key: &RsaKey, msg: &[u8]) -> Result<(Vec<u8>, Vec<u8>), EncryptionErrors>
{
    let random_key = rand::thread_rng().gen::<[u8; 32]>();
    //println!("key used: {:?}", random_key);
    let cipher = Aes256Gcm::new(Key::from_slice(&random_key));

    let nonce = Nonce::from_slice(b"unique nonce");

    Ok((rsa_key.encrypt(&random_key)?, cipher.encrypt(nonce, msg)?))
    // NOTE: handle these errors to avoid panics!
}

pub fn aes_decrypt(
    rsa_key: &RsaKey,
    encrypted_aes_key: &Vec<u8>,
    ciphertext: &Vec<u8>,
) -> Result<Vec<u8>, EncryptionErrors>
{
    let aes_key = rsa_key.decrypt(&encrypted_aes_key)?;
    //println!("key used: {:?}", aes_key);
    let cipher = Aes256Gcm::new(Key::from_slice(&aes_key));

    let nonce = Nonce::from_slice(b"unique nonce");

    Ok(cipher.decrypt(nonce, ciphertext.as_ref())?)
    // NOTE: handle this error to avoid panics!
}

pub fn get_sign_key<P>(path: P) -> Option<SigningKey>
where
    P: AsRef<std::path::Path>,
{
    SigningKey::from_bytes(&std::fs::read(path).ok()?).ok()
}

pub fn get_verify_key<P>(path: P) -> Option<VerifyingKey>
where
    P: AsRef<std::path::Path>,
{
    VerifyingKey::from_sec1_bytes(&std::fs::read(path).ok()?).ok()
}

pub fn get_verify_keys<P>(path_vec: &[P]) -> Vec<VerifyingKey>
where
    P: AsRef<std::path::Path>,
{
    let mut res = Vec::new();
    for path in path_vec
    {
        if let Some(key) = get_verify_key(path)
        {
            res.push(key);
        }
    }
    res
}
