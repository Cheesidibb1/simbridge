#[cfg(test)]
mod tests {
    use simbridge_shared::auth::crypto::*;

    #[test]
    fn test_pairing_code_generation() {
        let code = Crypto::generate_pairing_code();
        
        assert!(code.len() == 6); // 6-digit pairing code
        for c in code.chars() {
            assert!(c.is_ascii_digit());
        }
    }

    #[test]
    fn test_token_generation() {
        let token = Crypto::generate_session_token(123456);
        
        assert!(!token.is_empty());
        assert_eq!(token.len(), 32); // 128-bit token
    }

    #[test]
    fn test_hash_password() {
        let password = "testpassword123";
        let hashed = Crypto::hash_password(password);
        
        assert!(!hashed.is_empty());
        assert_eq!(hashed.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_verify_hash() {
        let password = "testpassword123";
        let hashed = Crypto::hash_password(password);
        
        let valid = Crypto::verify_password(password, &hashed);
        let invalid = Crypto::verify_password("wrongpassword", &hashed);

        assert!(valid);
        assert!(!invalid);
    }

    #[test]
    fn test_generate_challenge() {
        let challenge = Crypto::generate_challenge();
        
        assert!(challenge.len() == 32); // 256-bit challenge
    }

    #[test]
    fn test_verify_challenge_response() {
        let secret = b"test-secret-key";
        let challenge = Crypto::generate_challenge();
        
        let response = Crypto::compute_challenge_response(&challenge, &secret[..]);
        
        // Verify the response can be recomputed
        let recomputed = Crypto::verify_challenge_response(
            &response, 
            &challenge, 
            &secret[..]
        );

        assert!(recomputed);
    }

    #[test]
    fn test_generate_key_pair() {
        let (public_key, private_key) = Crypto::generate_key_pair();
        
        assert!(!public_key.is_empty());
        assert!(!private_key.is_empty());
    }

    #[test]
    fn test_encryption_decryption() {
        let plaintext = "Hello, SimBridge!";
        let key = [0u8; 32];
        
        let ciphertext = Crypto::encrypt(plaintext.as_bytes(), &key);
        assert!(!ciphertext.is_empty());

        let decrypted = Crypto::decrypt(&ciphertext, &key);
        assert_eq!(decrypted, plaintext.as_bytes());
    }

    #[test]
    fn test_aead_encryption() {
        let plaintext = b"Test data for AEAD encryption";
        let key = [0u8; 32];
        let nonce = [1u8; 12]; // 96-bit nonce
        
        let (ciphertext, tag) = Crypto::encrypt_aead(plaintext, &key, &nonce);
        
        assert_eq!(ciphertext.len(), plaintext.len());
        assert_eq!(tag.len(), 16); // 128-bit tag

        let decrypted = Crypto::decrypt_aead(&ciphertext, &tag, &key, &nonce);
        assert_eq!(&decrypted[..], plaintext);
    }
}
