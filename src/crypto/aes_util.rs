use aes::cipher::{generic_array::GenericArray, KeyIvInit};

pub type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
pub type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

pub fn initialize(shared_secret: &[u8]) -> (Aes128Cfb8Enc, Aes128Cfb8Dec) {
    let key = GenericArray::from_slice(shared_secret);

    let encryptor = Aes128Cfb8Enc::new(key, key);
    let decryptor = Aes128Cfb8Dec::new(key, key);

    (encryptor, decryptor)
}