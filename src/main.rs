/*
 * A password manager/utility
 * first starting with generating passwords
 * want to be able to handle those dumb contraints that are added on
 */

pub mod random;

use std::fs;
use std::collections::HashMap;
use std::path::PathBuf;
use base64::Engine;
use base64::engine::general_purpose;
use clap::{Parser, Subcommand};
use fernet::Fernet;
use rand::rngs::OsRng;
use scrypt::{Params, password_hash::SaltString};
use serde::{Serialize, Deserialize};
use anyhow;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    // vault/entry interaction
    Get(GetCommand),
    Add(AddCommand),
    Remove(RemoveCommand),
    // vault interaction
    Create(CreateCommand),
    Delete(DeleteCommand),
    // both
    List(ListCommand),
}

#[derive(Debug, Parser)]
struct GetCommand {
    #[arg(short, long, default_value="default")]
    vault: String,
    entry: String,
}

#[derive(Debug, Parser)]
struct AddCommand {
    #[arg(short, long, default_value="default")]
    vault: String,
    entry: String,
    username: String,
    password: String,
}

#[derive(Debug, Parser)]
struct RemoveCommand {
    #[arg(short, long, default_value="default")]
    vault: String,
    entry: String,
}

#[derive(Debug, Parser)]
struct CreateCommand {
    #[arg(default_value="default")]
    vault: String,
}

#[derive(Debug, Parser)]
struct DeleteCommand {
    vault: String,
}

#[derive(Debug, Parser)]
struct ListCommand {
    vault: Option<String>,
}

fn write_file(path: PathBuf, contents: String) -> Result<(), anyhow::Error> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn create_fernet(password: String, salt: String) -> Option<Fernet> {
    let mut out = vec![0u8;32];
    let _res = scrypt::scrypt(password.as_bytes(), salt.as_bytes(), &Params::new(16, 8, 1, 32).unwrap(), &mut out);

    let key = general_purpose::URL_SAFE.encode(&out);

    fernet::Fernet::new(&key)
}

fn write_encrypted(fernet: Fernet, path: PathBuf, data: VaultData) -> Result<(), anyhow::Error> {
    let content = serde_json::to_string(&data)?;
    let encrypted = fernet.encrypt(content.as_bytes());
    let res = write_file(path, encrypted)?;
    Ok(res)
}

fn read_encrypted(fernet: Fernet, path: PathBuf) -> Result<VaultData, anyhow::Error> {
    let content = fs::read_to_string(path)?;
    let decrypted = fernet.decrypt(&content)?;
    let str = String::from_utf8(decrypted)?;
    let val = serde_json::from_str(&str)?;
    Ok(val)
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
struct VaultData {
    data: HashMap<String, HashMap<String, String>>,
}

impl VaultData {
    fn new() -> Self {
        VaultData {data: HashMap::new()}
    }

    fn add(&mut self, entry: String, value: HashMap<String, String>) {
        self.data.insert(entry, value);
    }

    fn delete(&mut self, entry: &str) -> Option<HashMap<String, String>> {
        self.data.remove(entry)
    }

    fn get(&self, entry: &str) -> Option<&HashMap<String, String>> {
        self.data.get(entry)
    }

    fn entries(&self) -> Vec<String> {
        self.data.clone().into_keys().collect()
    }
}

#[derive(Clone)]
struct Vault {
    id: String,
    path: PathBuf,
    fernet: Fernet,
    data: VaultData,
}

impl Vault {
    fn new(id: String, password: String) -> Result<Vault, anyhow::Error> {
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

    fn create(id: String, password: String) -> Result<Vault, anyhow::Error> {
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

    fn get(&self, entry: &str) -> Option<&HashMap<String, String>> {
        self.data.get(entry)
    }

    fn set(&mut self, entry: String, username: String, password: String) -> Result<(), anyhow::Error> {
        let mut val = HashMap::new();
        val.insert("username".to_string(), username);
        val.insert("password".to_string(), password);
        self.data.add(entry, val);
        self.write()
    }

    fn remove(&mut self, entry: &str) -> Result<(), anyhow::Error> {
        self.data.delete(entry);
        self.write()
    }

    fn entries(&self) -> Vec<String> {
        self.data.entries()
    }

    fn write(&self) -> Result<(), anyhow::Error> {
        write_encrypted(self.fernet.clone(), self.path.clone(), self.data.clone())
    }
}

fn config_dir() -> PathBuf {
    let mut path = dirs::home_dir().expect("No home directory, not sure what to do");
    path.push(".turt");
    path
}

fn list_vaults() -> Result<Vec<String>, anyhow::Error> {
    let mut list = Vec::new();
    let path = config_dir();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            list.push(name);
        }
    }
    Ok(list)
}

fn main() {
    /*
    let x : Uppercase = rand::random();
    println!("{:?}", x);
    let y : Lowercase = rand::random();
    println!("{:?}", y);
    let z : Digit = rand::random();
    println!("{:?}", z);
    */

    // just always make sure .turt exists
    let _ = fs::create_dir_all(config_dir());

    let args = Cli::parse();

    match &args.command {
        Commands::Create(data) => {
            let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
            let confirm = rpassword::prompt_password("Confirm password: ").expect("Prompting for password failed");
            if confirm == password {
                match Vault::create(data.vault.clone(), password) {
                    Ok(_) => {
                        println!("Created new vault");
                    }
                    Err(e) => {
                        println!("Failed to create vault: {:?}", e);
                    }
                }
            } else {
                println!("Password's do not match.");
            }
        }
        Commands::Delete(data) => {
            let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
            match Vault::new(data.vault.clone(), password) {
                Ok(vault) => {
                    let _ = fs::remove_dir_all(vault.path.clone());
                    println!("Removed vault: {:?}", vault.path);
                }
                Err(e) => {
                    println!("Error decrypting vault: {:?}", e);
                }
            }
        }
        Commands::Get(data) => {
            let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
           
            match Vault::new(data.vault.clone(), password) {
                Ok(vault) => {
                    if let Some(info) = vault.get(&data.entry) {
                        for (key, value) in info.iter() {
                            println!("{}: {}", key, value);
                        }
                    } else {
                        println!("No entry for {}", data.entry);
                    }
                }
                Err(e) => {
                    println!("Error decrypting vault: {:?}", e);
                }
            }
        }
        Commands::Add(data) => {
            let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
            match Vault::new(data.vault.clone(), password) {
                Ok(mut vault) => {
                    if let Ok(_info) = vault.set(data.entry.clone(), data.username.clone(), data.password.clone()) {
                        println!("Created new entry for {}", data.entry)
                    } else {
                        println!("Failed to create entry for {}", data.entry);
                    }
                }
                Err(e) => {
                    println!("Error decrypting vault: {:?}", e);
                }
            }

        }
        Commands::Remove(data) => {
            let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
            match Vault::new(data.vault.clone(), password) {
                Ok(mut vault) => {
                    if let Ok(_info) = vault.remove(&data.entry) {
                        println!("Removed entry {}", data.entry)
                    } else {
                        println!("Failed to remove entry {}", data.entry);
                    }
                }
                Err(e) => {
                    println!("Error decrypting vault: {:?}", e);
                }
            }
        }
        Commands::List(data) => {
            match &data.vault {
                Some(name) => {
                    let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
                    match Vault::new(name.to_string(), password) {
                        Ok(vault) => {
                            println!("Entries in {}:", vault.id);
                            for item in vault.entries() {
                                println!(" - {}", item);
                            }
                        }
                        Err(e) => {
                            println!("Error decrypting vault: {:?}", e);
                        }
                    }
                }
                None => {
                    match list_vaults() {
                        Ok(list) => {
                            if list.len() == 0 {
                                println!("No vaults defined yet");
                            } else {
                                println!("Existing vaults:");
                                for vault in list {
                                    println!(" - {}", vault);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error listing vaults {:?}", e);
                        }
                    }
                }
            }
        }
    }
}
