use std::{path::PathBuf, fs};

use base64::{engine::general_purpose, Engine};
use fernet::Fernet;
use scrypt::Params;

use crate::vault::VaultData;

pub fn config_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("No home directory, not sure what to do");
    path.push(".turt");
    path
}

pub fn write_file(path: PathBuf, contents: String) -> Result<(), anyhow::Error> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

pub fn create_fernet(password: String, salt: String) -> Option<Fernet> {
    let mut out = vec![0u8;32];
    let _res = scrypt::scrypt(password.as_bytes(), salt.as_bytes(), &Params::new(16, 8, 1, 32).unwrap(), &mut out);

    let key = general_purpose::URL_SAFE.encode(&out);

    fernet::Fernet::new(&key)
}

pub fn write_encrypted(fernet: Fernet, path: PathBuf, data: VaultData) -> Result<(), anyhow::Error> {
    let content = serde_json::to_string(&data)?;
    let encrypted = fernet.encrypt(content.as_bytes());
    let res = write_file(path, encrypted)?;
    Ok(res)
}

pub fn read_encrypted(fernet: Fernet, path: PathBuf) -> Result<VaultData, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    let decrypted = fernet.decrypt(&content)?;
    let str = String::from_utf8(decrypted)?;
    let val = serde_json::from_str(&str)?;
    Ok(val)
}

