use chacha20poly1305::{AeadCore, Aead, ChaCha20Poly1305, KeyInit, Nonce};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use thiserror::Error;
use rand::RngCore;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptError(String),
    #[error("Decryption failed: {0}")]
    DecryptError(String),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Key generation failed: {0}")]
    KeyGenError(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;

pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptError(e.to_string()))?;
    let mut rng = rand::rngs::OsRng;
    let nonce = ChaCha20Poly1305::generate_nonce(&mut rng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|e| CryptoError::EncryptError(e.to_string()))?;
    let mut result = nonce.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(CryptoError::DecryptError("Ciphertext too short".to_string()));
    }
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::DecryptError(e.to_string()))?;
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let encrypted = &ciphertext[12..];
    cipher.decrypt(nonce, encrypted)
        .map_err(|_| CryptoError::DecryptError("Decryption failed".to_string()))
}

pub fn encrypt_to_base64(plaintext: &[u8], key: &[u8; 32]) -> Result<String> {
    let encrypted = encrypt(plaintext, key)?;
    Ok(BASE64.encode(&encrypted))
}

pub fn decrypt_from_base64(encoded: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
    let encrypted = BASE64.decode(encoded)
        .map_err(|e| CryptoError::DecryptError(format!("Invalid base64: {}", e)))?;
    decrypt(&encrypted, key)
}

pub fn encrypt_string(plaintext: &str, key: &[u8; 32]) -> Result<String> {
    let encrypted = encrypt(plaintext.as_bytes(), key)?;
    Ok(BASE64.encode(&encrypted))
}

pub fn decrypt_string(encrypted: &str, key: &[u8; 32]) -> Result<String> {
    let decrypted = decrypt_from_base64(encrypted, key)?;
    String::from_utf8(decrypted)
        .map_err(|e| CryptoError::DecryptError(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        key
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = test_key();
        let plaintext = "Secret message with unicode: 你好世界";
        
        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
