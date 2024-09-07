use cipher::{generic_array::GenericArray, BlockDecryptMut, BlockEncryptMut, KeyIvInit};

pub type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
pub type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

pub trait SimpleEncryptor {
    fn encrypt(&mut self, data: &[u8]) -> Vec<u8>;
}

pub trait SimpleDecryptor {
    fn decrypt(&mut self, data: &[u8]) -> Vec<u8>;
}

pub fn initialize(shared_secret: &[u8]) -> (Aes128Cfb8Enc, Aes128Cfb8Dec) {
    let encryptor = Aes128Cfb8Enc::new(shared_secret.into(), shared_secret.into());
    let decryptor = Aes128Cfb8Dec::new(shared_secret.into(), shared_secret.into());

    (encryptor, decryptor)
}

impl SimpleEncryptor for Aes128Cfb8Enc {
    fn encrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let mut encrypted_bytes: Vec<u8> = Vec::with_capacity(data.len());
        for b in data {
            let mut block = GenericArray::clone_from_slice(&[*b]);
            assert_eq!(b, &block.to_vec()[0]);
            self.encrypt_block_mut(&mut block);
            encrypted_bytes.extend_from_slice(&block);
        }
        
        encrypted_bytes
    }
}

impl SimpleDecryptor for Aes128Cfb8Dec {
    fn decrypt(&mut self, data: &[u8]) -> Vec<u8> {
        let mut decrypted_bytes: Vec<u8> = Vec::with_capacity(data.len());
        for b in data {
            let mut block = GenericArray::clone_from_slice(&[*b]);
            assert_eq!(b, &block.to_vec()[0]);
            self.decrypt_block_mut(&mut block);
            decrypted_bytes.extend_from_slice(&block);
        }
        
        decrypted_bytes
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_encrypt_trait() {
        // Generate random shared secret
        let mut rng = rand::thread_rng();
        let shared_secret: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        println!("Shared secret: {:x?}", shared_secret);

        // Initialize encryptor and decryptor
        let (mut encryptor1, _) = initialize(&shared_secret);    
        let mut encryptor2 = encryptor1.clone();

        // Generate a single block of data to encrypt and decrypt
        let data1: Vec<u8> = (0..16).map(|_| rng.gen()).collect();    
        let data2 = data1.clone();
        println!("Data ({} bytes): {:x?}", data1.len(), data1);

        let mut encrypted_bytes1: Vec<u8> = Vec::new();
        for b in data1 {
            let mut block = GenericArray::clone_from_slice(&[b]);
            encryptor1.encrypt_block_mut(&mut block);
            encrypted_bytes1.extend_from_slice(block.as_slice());
        }

        println!("Encrypted data #1 ({} bytes): {:x?}", encrypted_bytes1.len(), encrypted_bytes1);

        let encrypted_bytes2 = encryptor2.encrypt(&data2);
        println!("Encrypted data #2 ({} bytes): {:x?}", encrypted_bytes2.len(), encrypted_bytes2);

        assert_eq!(encrypted_bytes1, encrypted_bytes2);
    }

    #[test]
    fn test_decrypt_trait() {
        // Generate random shared secret
        let mut rng = rand::thread_rng();
        let shared_secret: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        println!("Shared secret: {:x?}", shared_secret);

        // Initialize encryptor and decryptor
        let (_, mut decryptor1) = initialize(&shared_secret);    
        let mut decryptor2 = decryptor1.clone();

        // Generate a single block of data to encrypt and decrypt
        let data1: Vec<u8> = (0..16).map(|_| rng.gen()).collect();    
        let data2 = data1.clone();
        println!("Data ({} bytes): {:x?}", data1.len(), data1);

        let mut decrypted_bytes1: Vec<u8> = Vec::new();
        for b in data1 {
            let mut block = GenericArray::clone_from_slice(&[b]);
            decryptor1.decrypt_block_mut(&mut block);
            decrypted_bytes1.extend_from_slice(block.as_slice());
        }

        println!("Decrypted data #1 ({} bytes): {:x?}", decrypted_bytes1.len(), decrypted_bytes1);

        let decrypted_bytes2 = decryptor2.decrypt(&data2);
        println!("Decrypted data #2 ({} bytes): {:x?}", decrypted_bytes2.len(), decrypted_bytes2);

        assert_eq!(decrypted_bytes1, decrypted_bytes2);
    }

    #[test]
    fn test_once() {
        // Generate random shared secret
        let mut rng = rand::thread_rng();
        let shared_secret: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        println!("Shared secret: {:x?}", shared_secret);

        // Initialize encryptor and decryptor
        let (mut encryptor, mut decryptor) = initialize(&shared_secret);    

        // Generate a single block of data to encrypt and decrypt
        let data: Vec<u8> = (0..16).map(|_| rng.gen()).collect();    
        println!("Data ({} bytes): {:x?}", data.len(), data);

        let encrypted_bytes = encryptor.encrypt(&data);
        println!("Encrypted data ({} bytes): {:x?}", encrypted_bytes.len(), encrypted_bytes);

        let decrypted_bytes = decryptor.decrypt(&encrypted_bytes);
        println!("Decrypted data ({} bytes): {:x?}", decrypted_bytes.len(), decrypted_bytes);

        assert_eq!(data, decrypted_bytes);
    }   

    #[test]
    fn test_several() {
        // Key and IV (Initialization Vector)
        let mut rng = rand::thread_rng();
        let shared_secret: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        println!("Shared secret: {:x?}", shared_secret);

        // Initialize encryptor and decryptor
        let (mut encryptor, mut decryptor) = initialize(&shared_secret);    

        // Simulate a single block of data to encrypt and decrypt
        for i in 0..5 {
            println!("Test #{}", i);
            let data: Vec<u8> = (0..16).map(|_| rng.gen()).collect();    
            println!("  Data ({} bytes): {:x?}", data.len(), data);

            let encrypted_bytes = encryptor.encrypt(&data);
            println!("  Encrypted data ({} bytes): {:x?}", encrypted_bytes.len(), encrypted_bytes);

            let decrypted_bytes = decryptor.decrypt(&encrypted_bytes);
            println!("  Decrypted data ({} bytes): {:x?}", decrypted_bytes.len(), decrypted_bytes);

            assert_eq!(data, decrypted_bytes);
        }
    }   

    #[test]
    fn test_long_data() {
        // Generate random shared secret
        let mut rng = rand::thread_rng();
        let shared_secret: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        println!("Shared secret: {:x?}", shared_secret);

        // Initialize encryptor and decryptor
        let (mut encryptor, mut decryptor) = initialize(&shared_secret);    

        // Generate a single block of data to encrypt and decrypt
        let data: Vec<u8> = (0..256).map(|_| rng.gen()).collect();    
        println!("Data ({} bytes): {:x?}", data.len(), data);

        let encrypted_bytes = encryptor.encrypt(&data);
        println!("Encrypted data ({} bytes): {:x?}", encrypted_bytes.len(), encrypted_bytes);

        let decrypted_bytes = decryptor.decrypt(&encrypted_bytes);
        println!("Decrypted data ({} bytes): {:x?}", decrypted_bytes.len(), decrypted_bytes);

        assert_eq!(data, decrypted_bytes);
    }
}