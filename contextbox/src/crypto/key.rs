use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const KEY_DIR: &str = "contextbox";
const KEY_FILE: &str = "key.txt";

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join(KEY_DIR))
        .unwrap_or_else(|| PathBuf::from("~/.config/contextbox"))
}

pub fn get_key_path() -> PathBuf {
    get_config_dir().join(KEY_FILE)
}

pub fn ensure_config_dir() -> io::Result<PathBuf> {
    let dir = get_config_dir();
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn generate_key() -> [u8; 32] {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut key);
    key
}

pub fn key_to_base64(key: &[u8; 32]) -> String {
    BASE64.encode(key)
}

pub fn key_from_base64(encoded: &str) -> Result<[u8; 32], String> {
    let decoded = BASE64.decode(encoded)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    
    if decoded.len() != 32 {
        return Err(format!("Invalid key length: {} (expected 32)", decoded.len()));
    }
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded);
    Ok(key)
}

pub fn ensure_key() -> io::Result<([u8; 32], bool)> {
    let key_path = get_key_path();
    
    if key_path.exists() {
        let content = fs::read_to_string(&key_path)?;
        let content = content.trim();
        
        match key_from_base64(content) {
            Ok(key) => return Ok((key, false)),
            Err(e) => {
                eprintln!("Warning: Invalid key file: {}", e);
                eprintln!("Generating new key...");
            }
        }
    }
    
    ensure_config_dir()?;
    
    let key = generate_key();
    let key_b64 = key_to_base64(&key);
    
    let mut file = fs::File::create(&key_path)?;
    writeln!(file, "{}", key_b64)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms)?;
    }
    
    Ok((key, true))
}

pub fn load_key() -> Result<[u8; 32], String> {
    let key_path = get_key_path();
    
    if !key_path.exists() {
        return Err(format!(
            "Key not found at {}. Run 'cb keygen' first.",
            key_path.display()
        ));
    }
    
    let content = fs::read_to_string(&key_path)
        .map_err(|e| format!("Failed to read key: {}", e))?;
    
    let content = content.trim();
    key_from_base64(content)
}

pub fn key_exists() -> bool {
    get_key_path().exists()
}
