/*
 * A password manager/utility
 * first starting with generating passwords
 * want to be able to handle those dumb contraints that are added on
 */

pub mod password;
pub mod vault;
pub mod utils;

use core::time;
use std::{fs, thread};
use arboard::{Clipboard, SetExtLinux};
use clap::{Parser, Subcommand};
use password::{Password, generic, Choice};
use anyhow;
use utils::config_dir;
use vault::Vault;

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
#[command(about="Get an entry from a vault, passwords are copied to clipboard")]
struct GetCommand {
    #[arg(short, long, default_value="default", help="The vault to access")]
    vault: String,
    #[arg(help="The entry to get the information for")]
    entry: String,
    #[arg(short, long, default_value_t=10, help="How long to hold the password in the clipboard for")]
    duration: u64,
}

#[derive(Debug, Parser)]
#[command(about="Add a username and password to the vault. Password is either specified manually or given a specification and generated automatically")]
struct AddCommand {
    #[arg(short, long, default_value="default", help="The vault to access")]
    vault: String,
    #[arg(help="The entry to add")]
    entry: String,
    #[arg(help="The username for the entry")]
    username: String,
    #[arg(help="(Optional) Manually specified password if not generating the password")]
    password: Option<String>,
    #[arg(long, help="the allowed characters for the generated password, defaults to a reasonable group of ascii characters")]
    allowed: Option<String>,
    #[arg(long, default_value_t=32, help="length of the generated password")]
    length: u32,
    #[arg(long, help="pattern for the generated password (a subset of 'digit+upper+lower+alpha+symbol')")]
    pattern: Option<String>,
}

#[derive(Debug, Parser)]
#[command(about="Remove an entry from a vault")]
struct RemoveCommand {
    #[arg(short, long, default_value="default", help="The vault to access")]
    vault: String,
    #[arg(help="The entry to remove")]
    entry: String,
}

#[derive(Debug, Parser)]
#[command(about="Create a new vault")]
struct CreateCommand {
    #[arg(default_value="default", help="The vault to delete")]
    vault: String,
}

#[derive(Debug, Parser)]
#[command(about="Delete a vault")]
struct DeleteCommand {
    #[arg(help="The vault to delete")]
    vault: String,
}

#[derive(Debug, Parser)]
#[command(about="Either list the existing vaults or the entries in a vault")]
struct ListCommand {
    #[arg(help="The vault to access")]
    vault: Option<String>,
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

fn new_vault(vault: String) -> Vault {
    let password = rpassword::prompt_password("Vault password: ").expect("Prompting for password failed");
    match Vault::new(vault.clone(), password) {
        Ok(v) => v,
        Err(e) => {
            panic!("Error decrypting vault {}: {:?}", vault, e);
        }
    }
}

fn main() {
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
            let vault = new_vault(data.vault.clone());
            let _ = fs::remove_dir_all(vault.path.clone());
            println!("Removed vault: {:?}", vault.path);
        }
        Commands::Get(data) => {
            let vault = new_vault(data.vault.clone());
            if let Some(info) = vault.get(&data.entry) {
                for (key, value) in info.iter() {
                    if key != "password" {
                        println!("{}: {}", key, value);
                    }
                }
                if let Some(password) = info.get("password") {
                    let dur = data.duration;
                    let mut threads = vec![];
                    let mut clip = Clipboard::new().unwrap();
                    let orig = clip.get_text().unwrap_or("".to_string());
                    let p = password.clone();
                    threads.push(thread::spawn(move || {
                        println!("Copying password to clipboard for {} seconds.", dur);
                        let _ = Clipboard::new().unwrap().set().wait().text(p.to_string());
                    }));
                    threads.push(thread::spawn(move || {
                        let wait_time = time::Duration::from_millis(dur*1000);
                        thread::sleep(wait_time);
                        let _ = Clipboard::new().unwrap().set_text(orig);
                        // lol, whatever
                        let wait_time = time::Duration::from_millis(1000);
                        thread::sleep(wait_time);
                    }));
                    for t in threads {
                        let _ = t.join();
                    }
                }
            } else {
                println!("No entry for {}", data.entry);
            }
        }
        Commands::Add(data) => {
            let mut vault = new_vault(data.vault.clone());
            match &data.password {
                Some(p) => {
                    if let Ok(_info) = vault.set(data.entry.clone(), data.username.clone(), p.clone()) {
                        println!("Created new entry for {}", data.entry)
                    } else {
                        println!("Failed to create entry for {}", data.entry);
                    }
                }
                None => {
                    let length = data.length;
                    let allowed = match &data.allowed {
                        Some(chars) => Choice::new(chars.chars().collect()).unwrap(),
                        None => generic(),
                    };
                    let pattern = data.pattern.clone().unwrap_or("".to_string());
                    if let Some(spec) = Password::from_spec(allowed, length, pattern.clone()) {
                        if let Ok(_info) = vault.set_password(data.entry.clone(), data.username.clone(), spec) {
                            println!("Created new entry for {}", data.entry)
                        } else {
                            println!("Failed to create entry for {}", data.entry);
                        }
                    } else {
                        println!("Invalid password specification");
                    }
                }
            }
        }
        Commands::Remove(data) => {
            let mut vault = new_vault(data.vault.clone());
            if let Ok(_info) = vault.remove(&data.entry) {
                println!("Removed entry {}", data.entry)
            } else {
                println!("Failed to remove entry {}", data.entry);
            }
        }
        Commands::List(data) => {
            match &data.vault {
                Some(name) => {
                    let vault = new_vault(name.to_string());
                    let entries = vault.entries();
                    if entries.len() == 0 {
                        println!("No entries for {} yet", vault.id);
                    } else {
                        println!("Entries in {}:", vault.id);
                        for item in vault.entries() {
                            println!(" - {}", item);
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
