use std::path::PathBuf;

use super::LoadError;

pub(crate) fn path_from_env_or_home(env: &str, file_name: &str) -> Result<PathBuf, LoadError> {
    match std::env::var(env) {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(_) => dirs::home_dir()
            .map(|path| path.join(".qcs").join(file_name))
            .ok_or_else(|| LoadError::HomeDirError {
                env: env.to_string(),
            }),
    }
}
