use rand::thread_rng;
use rsa::{RsaPrivateKey, RsaPublicKey};

const BITS: usize = 1024;

pub fn generate_rsa_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    let mut rng = thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, BITS).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);
    (priv_key, pub_key)
}