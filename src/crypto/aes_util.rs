use aes::{cipher::{generic_array::GenericArray, KeyIvInit, AsyncStreamCipher}};

type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

pub fn encrypt(shared_secret: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let key = *GenericArray::from_slice(&shared_secret);
    let iv = *GenericArray::from_slice(&[0u8; 16]); // Your IV (Initialization Vector)

    // Create the AES/CFB8 cipher instance
    let mut buf = plaintext.to_vec();
    Aes128Cfb8Enc::new(&key.into(), &iv.into()).encrypt(&mut buf);

    buf
}