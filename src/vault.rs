
use crate::utils::{write_encrypted, read_encrypted, write_file, create_fernet};
use std::{fs, fmt::Display};
use crate::utils::config_dir;
use std::collections::HashMap;
use std::path::PathBuf;
use fernet::Fernet;
use crate::password::Password;
use rand::rngs::OsRng;
use scrypt::password_hash::SaltString;
use serde::{Serialize, Deserialize};
use anyhow;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum VaultItem {
    Generic(String),
    GeneratedPassword(String, Password),
}

impl Display for VaultItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultItem::Generic(s) => s.fmt(f),
            VaultItem::GeneratedPassword(s, _) => s.fmt(f),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VaultData {
    data: HashMap<String, HashMap<String, VaultItem>>,
}

impl VaultData {
    fn new() -> Self {
        VaultData {data: HashMap::new()}
    }

    fn add(&mut self, entry: String, value: HashMap<String, VaultItem>) {
        self.data.insert(entry, value);
    }

    fn delete(&mut self, entry: &str) -> Option<HashMap<String, VaultItem>> {
        self.data.remove(entry)
    }

    fn get(&self, entry: &str) -> Option<&HashMap<String, VaultItem>> {
        self.data.get(entry)
    }

    fn entries(&self) -> Vec<String> {
        self.data.clone().into_keys().collect()
    }
}

#[derive(Clone)]
pub struct Vault {
    pub id: String,
    pub path: PathBuf,
    fernet: Fernet,
    pub data: VaultData,
}

impl Vault {
    fn check(id: String) -> bool {
        let mut dir = config_dir();
        dir.push(id.clone());
        let mut data_file = dir.clone();
        data_file.push("data");
        data_file.set_extension("json");

        let mut salt_file = dir.clone();
        salt_file.push("salt");
        salt_file.set_extension("txt");
        dir.exists() && data_file.exists() && salt_file.exists()
    }

    pub fn new(id: String, password: String) -> Result<Vault, anyhow::Error> {
        let mut dir = config_dir();
        dir.push(id.clone());
        let mut data_file = dir.clone();
        data_file.push("data");
        data_file.set_extension("json");

        let mut salt_file = dir.clone();
        salt_file.push("salt");
        salt_file.set_extension("txt");

        let salt = fs::read_to_string(salt_file)?;

        let fernet = create_fernet(password.to_string(), salt.to_string()).expect("Failed to setup encryption");
        let data = read_encrypted(fernet.clone(), data_file.clone())?; 
        Ok(Vault { id, path: data_file, fernet, data })
    }

    pub fn create(id: String, password: String) -> Result<Vault, anyhow::Error> {
        let mut dir = config_dir();
        dir.push(id.clone());
        let mut data_file = dir.clone();
        data_file.push("data");
        data_file.set_extension("json");

        let mut salt_file = dir.clone();
        salt_file.push("salt");
        salt_file.set_extension("txt");

        let salt = SaltString::generate(&mut OsRng); 
        write_file(salt_file, salt.to_string())?;

        let fernet = create_fernet(password.to_string(), salt.to_string()).expect("Failed to setup encryption");
        let data = VaultData::new();
        let vault = Vault { id, path: data_file, fernet, data };
        vault.write()?;
        Ok(vault)
    }

    pub fn get(&self, entry: &str) -> Option<&HashMap<String, VaultItem>> {
        self.data.get(entry)
    }

    pub fn set(&mut self, entry: String, username: String, password: String) -> Result<(), anyhow::Error> {
        let mut val = HashMap::new();
        val.insert("username".to_string(), VaultItem::Generic(username));
        val.insert("password".to_string(), VaultItem::Generic(password));
        self.data.add(entry, val);
        self.write()
    }

    pub fn set_password(&mut self, entry: String, username: String, password: Password) -> Result<(), anyhow::Error> {
        let mut val = HashMap::new();
        val.insert("username".to_string(), VaultItem::Generic(username));
        let password_str = password.generate();
        val.insert("password".to_string(), VaultItem::GeneratedPassword(password_str, password));
        self.data.add(entry, val);
        self.write()
    }

    pub fn remove(&mut self, entry: &str) -> Result<(), anyhow::Error> {
        self.data.delete(entry);
        self.write()
    }

    pub fn entries(&self) -> Vec<String> {
        self.data.entries()
    }

    pub fn write(&self) -> Result<(), anyhow::Error> {
        write_encrypted(self.fernet.clone(), self.path.clone(), self.data.clone())
    }
}


