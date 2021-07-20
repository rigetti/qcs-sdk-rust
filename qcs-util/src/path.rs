use std::path::PathBuf;

use eyre::{eyre, Result};

pub(crate) fn path_from_env_or_home(env: &str, file_name: &str) -> Result<PathBuf> {
    match std::env::var(env) {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(_) => dirs::home_dir()
            .map(|path| path.join(".qcs").join(file_name))
            .ok_or_else(||
                eyre!("Failed to determine home directory. You can use an explicit path by setting the {} environment variable", env)
            ),
    }
}
