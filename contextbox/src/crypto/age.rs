use age_core::identity::Identity;
use age_core::keys::SecretKey;
use age_core::primitives::{aead_decrypt, aead_encrypt};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use thiserror::Error;

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
    let secret_key = SecretKey::from(key);
    let nonce = generate_nonce();
    
    let ciphertext = aead_encrypt(&secret_key, &nonce, plaintext, &[])
        .map_err(|e| CryptoError::EncryptError(e.to_string()))?;
    
    let mut result = Vec::with_capacity(12 + nonce.len() + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(CryptoError::DecryptError("Ciphertext too short".to_string()));
    }
    
    let secret_key = SecretKey::from(key);
    
    let nonce = &ciphertext[..12];
    let encrypted = &ciphertext[12..];
    
    let plaintext = aead_decrypt(&secret_key, nonce, encrypted, &[])
        .map_err(|e| CryptoError::DecryptError(e.to_string()))?;
    
    Ok(plaintext)
}

fn generate_nonce() -> [u8; 12] {
    use rand::RngCore;
    let mut nonce = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    nonce
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

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"Hello, World!";
        
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = generate_key();
        let plaintext = "Secret message with unicode: 你好世界";
        
        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
