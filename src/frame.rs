use bytes::{Buf, BytesMut};
use rand::Rng;
use serde::{self, Deserialize, Serialize};
use std::io::{Error, ErrorKind};
use std::sync::atomic::AtomicPtr;

use tokio::io::Result;
use tokio::net::TcpStream;

use super::encryption::{self, RsaKey, SignVerify};
use super::log::{log, Level};
use super::message::Message;

type SessionId = Option<i128>;

use k256::ecdsa::{
    signature::{Signer, Verifier},
    Signature,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum Frame
{
    String(String, SessionId),
    Vec(Vec<Frame>, SessionId),
    Message(Message, SessionId),
    KeyShare(String),
    SessionId(SessionId),
}

pub struct Connection
{
    stream: TcpStream,
    buffer: BytesMut,
    key: RsaKey,
    dig_sign: SignVerify,
    pub session_id: SessionId, // ... other fields here
}

impl Connection
{
    pub fn new(stream: TcpStream, key: RsaKey, dig_sign: SignVerify) -> Connection
    {
        Connection {
            stream,
            // Allocate the buffer with 4kb of capacity.
            buffer: BytesMut::with_capacity(4096),
            key,
            dig_sign,
            session_id: None,
        }
    }
    /// Write a frame to the connection.
    pub async fn write_frame(&self, frame: &Frame) -> Result<()>
    {
        self.write_frame_with_key(frame, &self.key, true).await
    }

    async fn write_frame_with_key(&self, frame: &Frame, key: &RsaKey, sign: bool) -> Result<()>
    {
        let parse_frame = ron::to_string(frame).unwrap();

        let parse_frame_encrypted = match encryption::aes_encrypt(key, parse_frame.as_bytes())
        {
            Ok(result) => result,
            Err(err) => panic!("could not encrypt the frame with respective key"),
        };

        let parse_frame_signature: Option<Signature>;
        if sign
        {
            parse_frame_signature = match &self.dig_sign
            {
                SignVerify::Sign(signer) => Some(signer.sign(parse_frame.as_bytes())),
                SignVerify::Verify(_) => panic!("cannot sign with verify key"),
                SignVerify::Both(signer, _) => Some(signer.sign(parse_frame.as_bytes())),
                SignVerify::MultiVerify(_) => panic!("cannot sign with verify key"),
            };
        }
        else
        {
            parse_frame_signature = None;
        }

        let parse_frame_encrypted_signed = (
            parse_frame_encrypted.0,
            parse_frame_encrypted.1,
            parse_frame_signature,
        );

        //there's a better solution. This is only needed because the write buffer has
        //to end with '\0' with the current logic
        let final_buf = ron::to_string(&parse_frame_encrypted_signed).unwrap() + "\0";

        loop
        {
            self.stream.writable().await?;
            let mut bytes_written = 0;
            match self
                .stream
                .try_write(&final_buf[bytes_written..].as_bytes())
            {
                Ok(n) =>
                {
                    bytes_written += n;
                    if bytes_written == final_buf.len()
                    {
                        break;
                    }
                }

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,

                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
        // implementation here
    }
    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> Result<Frame>
    {
        let key_ptr = AtomicPtr::new(&mut self.key);
        let key;
        //override the borrow checker, so it's possible to use &mut self and &self.key
        //it's fine because self.key is never mutated here
        unsafe {
            key = &*key_ptr.into_inner();
        }

        self.read_frame_with_key(key, true).await
    }

    async fn read_frame_with_key(&mut self, key: &RsaKey, signed: bool) -> Result<Frame>
    {
        loop
        {
            //try get frame from buffer
            if let Some(frame) = self.parse_frame(key, signed)
            {
                return Ok(frame);
            }

            self.stream.readable().await?;
            match self.stream.try_read_buf(&mut self.buffer)
            {
                Ok(0) => return Err(Error::from(ErrorKind::ConnectionReset)),

                Ok(_) => log(
                    Level::Debug,
                    &format!("current buffer: {}", String::from_utf8_lossy(&self.buffer)),
                ),

                Err(error) if error.kind() == ErrorKind::WouldBlock => continue,

                Err(error) => return Err(error),
            };
        }
    }

    fn parse_frame(&mut self, key: &RsaKey, signed: bool) -> Option<Frame>
    {
        let mut frame_len = None;
        let mut index = 0;

        for i in self.buffer.chunk()
        {
            index += 1;
            //frame comes invalid if there's \0 where it shouldn't
            if *i == '\0' as u8
            {
                frame_len = Some(index);
                break;
            }
        }
        if frame_len.is_none()
        {
            return None; //unfinished
        }

        let frame_len = frame_len.unwrap();

        let (frame, _) = self.buffer.chunk().split_at(frame_len - 1);

        let res = (|| {
            let frame_signed = &ron::from_str::<(Vec<u8>, Vec<u8>, Option<Signature>)>(
                &String::from_utf8_lossy(&frame),
            )
            .ok()?; //wrong parse

            let frame = encryption::aes_decrypt(key, &frame_signed.0, &frame_signed.1).ok()?; //wrong encryption

            if signed
            {
                let signature = frame_signed.2?;
                match &self.dig_sign
                {
                    SignVerify::Sign(_) => panic!("cannot verify with signer key"),
                    SignVerify::Verify(ver) => ver.verify(&frame, &signature),
                    SignVerify::Both(_, ver) => ver.verify(&frame, &signature),
                    SignVerify::MultiVerify(ver_vec) =>
                    {
                        let mut res = Err(k256::ecdsa::signature::Error::new());
                        //TODO: lazy iteration, because it's usually the same value in the vec
                        //that's used in a connection
                        for ver in ver_vec
                        {
                            res = ver.verify(&frame, &signature);
                            if res.is_ok()
                            {
                                break;
                            }
                        }
                        res
                    }
                }
                .ok()?; //cannot validate
            }

            ron::from_str(&String::from_utf8_lossy(&frame)).ok() //invalid frame
        })();

        self.buffer.advance(frame_len);

        return res;
        //NOTE: returns none if frame is invalid
    }

    pub async fn authenticate(mut self, auth: Auth) -> Result<Self>
    {
        match auth
        {
            Auth::Client =>
            {
                let rsa_priv_key;
                let pub_key_string;
                {
                    //create a private rsa key.
                    let mut rng = rand::thread_rng();
                    let bits = 512;
                    let priv_key =
                        rsa::RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");

                    //generate public key
                    let pub_key = rsa::RsaPublicKey::from(&priv_key);
                    pub_key_string = rsa::pkcs1::EncodeRsaPublicKey::to_pkcs1_pem(
                        &pub_key,
                        rsa::pkcs8::LineEnding::LF,
                    )
                    .unwrap();

                    rsa_priv_key = RsaKey::Private(priv_key);
                }

                //write frame with respective public key (KeyShare)
                self.write_frame(&Frame::KeyShare(pub_key_string)).await?;

                //read frame using created private rsa (SessionId) signed = false
                let id = match Connection::read_frame_with_key(&mut self, &rsa_priv_key, false)
                    .await?
                {
                    Frame::String(_, _) => todo!(),
                    Frame::Vec(_, _) => todo!(),
                    Frame::Message(_, _) => todo!(),
                    Frame::KeyShare(_) => todo!(),
                    Frame::SessionId(id) => id,
                };

                //set session id
                self.session_id = id;

                Ok(self)
            }
            Auth::Host =>
            {
                //generate a random session id
                let id = rand::thread_rng().gen();
                self.session_id = Some(id);
                //read frame (KeyShare) signed = true
                let key = match self.read_frame().await?
                {
                    Frame::String(_, _) => todo!(),
                    Frame::Vec(_, _) => todo!(),
                    Frame::Message(_, _) => todo!(),
                    Frame::KeyShare(key) => key,
                    Frame::SessionId(_) => todo!(),
                };
                let key =
                    <rsa::RsaPublicKey as rsa::pkcs1::DecodeRsaPublicKey>::from_pkcs1_pem(&key)
                        .unwrap();

                //write frame using the KeyShare key (SessionId) signed = false
                self.write_frame_with_key(&Frame::SessionId(Some(id)), &RsaKey::Public(key), false)
                    .await?;

                Ok(self)
            }
        }
    }
}

pub enum Auth
{
    Client,
    Host,
}
