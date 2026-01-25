
pub fn configuration_path() -> std::path::PathBuf {
    match dirs::config_local_dir() {
        Some(path) => path.join(crate::APPLICATION_NAME),
        None => {
            let hidden_path = format!(".{}", crate::APPLICATION_NAME);

            match dirs::home_dir() {
                Some(path) => {
                    path.join(&hidden_path)
                }
                None => std::path::PathBuf::from(&hidden_path)
            }
        },
    }
}

pub fn backup_path() -> std::path::PathBuf {
    match dirs::state_dir() {
        Some(path) => path.join(crate::APPLICATION_NAME),
        None => configuration_path(),
    }
}
