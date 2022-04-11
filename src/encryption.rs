use rsa::{errors::Error, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

pub enum RsaKey
{
    Private(RsaPrivateKey),
    Public(RsaPublicKey),
}

impl RsaKey
{
    pub fn encrypt(&self, msg: &[u8]) -> Vec<u8>
    {
        let mut rng = rand::thread_rng();
        match self
        {
            RsaKey::Private(key) => key
                .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
                .unwrap(),
            RsaKey::Public(key) => key
                .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
                .unwrap(),
        }
    }

    pub fn decrypt(&self, cypher: &[u8]) -> Result<Vec<u8>, Error>
    {
        match self
        {
            RsaKey::Private(key) => key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), cypher),
            RsaKey::Public(_) => Err(Error::Decryption),
        }
    }
}
