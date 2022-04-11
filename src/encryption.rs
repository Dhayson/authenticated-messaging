use rsa::{errors::Error, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

pub enum Key
{
    Private(RsaPrivateKey),
    Public(RsaPublicKey),
}

impl Key
{
    pub fn encrypt(&self, msg: &[u8]) -> Vec<u8>
    {
        let mut rng = rand::thread_rng();
        match self
        {
            Key::Private(key) => key
                .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
                .unwrap(),
            Key::Public(key) => key
                .encrypt(&mut rng, PaddingScheme::new_pkcs1v15_encrypt(), msg)
                .unwrap(),
        }
    }

    pub fn decrypt(&self, cypher: &[u8]) -> Result<Vec<u8>, Error>
    {
        match self
        {
            Key::Private(key) => key.decrypt(PaddingScheme::new_pkcs1v15_encrypt(), cypher),
            Key::Public(_) => Err(Error::Decryption),
        }
    }
}
