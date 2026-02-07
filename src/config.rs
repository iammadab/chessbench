use serde::Deserialize;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfigFile {
    pub engine: Vec<EngineConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub id: String,
    pub path: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub enum ConfigError {
    EmptyEngineList,
    EmptyId,
    EmptyPath,
    DuplicateId(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::EmptyEngineList => write!(f, "engine list is empty"),
            ConfigError::EmptyId => write!(f, "engine id is empty"),
            ConfigError::EmptyPath => write!(f, "engine path is empty"),
            ConfigError::DuplicateId(id) => write!(f, "duplicate engine id: {id}"),
        }
    }
}

impl EngineConfigFile {
    pub fn from_str(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.engine.is_empty() {
            return Err(ConfigError::EmptyEngineList);
        }

        let mut seen = HashSet::new();
        for entry in &self.engine {
            if entry.id.trim().is_empty() {
                return Err(ConfigError::EmptyId);
            }

            if entry.path.as_os_str().is_empty() {
                return Err(ConfigError::EmptyPath);
            }

            if !seen.insert(entry.id.clone()) {
                return Err(ConfigError::DuplicateId(entry.id.clone()));
            }
        }

        Ok(())
    }
}
