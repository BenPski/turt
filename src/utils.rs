use std::path::PathBuf;

use dirs::home_dir;

pub fn config_dir() -> Option<PathBuf> {
    if let Some(mut home) = home_dir() {
        home.push(".turt");
        Some(home)
    } else {
        None
    }
}
