// Cryptographic utilities

use rand::Rng;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, AeadCore, OsRng}};
use base64::{Engine as _, engine::general_purpose};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Cryptographic utilities
pub struct Crypto;

impl Crypto {
    /// Generate a random string of specified length
    pub fn random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Generate a random bytes array
    pub fn random_bytes(length: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill(&mut bytes[..]);
        bytes
    }

    /// Generate a random pairing code
    pub fn generate_pairing_code() -> String {
        format!("{}-{}-{}", 
            Self::random_string(4).to_uppercase(),
            Self::random_string(4).to_uppercase(),
            Self::random_string(4).to_uppercase()
        )
    }

    /// Compute SHA-256 hash
    pub fn sha256(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Compute HMAC-SHA256
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<String, CryptoError> {
        let mut mac = HmacSha256::new_from_slice(key)
            .map_err(|_| CryptoError::InvalidKey)?;
        mac.update(data);
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<String, CryptoError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
        
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(key: &[u8], ciphertext: &str) -> Result<Vec<u8>, CryptoError> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);
        
        let data = general_purpose::STANDARD.decode(ciphertext)
            .map_err(|e| CryptoError::Base64Error(e.to_string()))?;
        
        if data.len() < 12 {
            return Err(CryptoError::InvalidCiphertext);
        }
        
        let (nonce, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionError(e.to_string()))
    }

    /// Generate a secure key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    }

    /// Derive a key from a password using PBKDF2
    pub fn derive_key(password: &str, salt: &[u8], iterations: u32) -> Vec<u8> {
        use pbkdf2::pbkdf2_hmac;
        
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
        key.to_vec()
    }
}

/// Cryptographic errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Invalid key")]
    InvalidKey,
    
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    
    #[error("Invalid ciphertext")]
    InvalidCiphertext,
    
    #[error("Base64 error: {0}")]
    Base64Error(String),
}
