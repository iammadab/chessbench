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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_engine_config_with_optional_args() {
        let input = r#"
            [[engine]]
            id = "stockfish-16"
            path = "/opt/stockfish"
            args = ["-threads", "4"]

            [[engine]]
            id = "lc0-0.30"
            path = "/opt/lc0"
        "#;

        let config = EngineConfigFile::from_str(input).expect("parse config");
        config.validate().expect("validate config");

        assert_eq!(config.engine.len(), 2);
        assert_eq!(config.engine[0].id, "stockfish-16");
        assert_eq!(config.engine[0].path, PathBuf::from("/opt/stockfish"));
        assert_eq!(config.engine[0].args, vec!["-threads", "4"]);
        assert_eq!(config.engine[0].working_dir, None);

        assert_eq!(config.engine[1].id, "lc0-0.30");
        assert_eq!(config.engine[1].path, PathBuf::from("/opt/lc0"));
        assert!(config.engine[1].args.is_empty());
    }

    #[test]
    fn rejects_duplicate_ids() {
        let input = r#"
            [[engine]]
            id = "dup"
            path = "/opt/one"

            [[engine]]
            id = "dup"
            path = "/opt/two"
        "#;

        let config = EngineConfigFile::from_str(input).expect("parse config");
        let err = config.validate().expect_err("validate should fail");

        match err {
            ConfigError::DuplicateId(id) => assert_eq!(id, "dup"),
            _ => panic!("expected duplicate id error"),
        }
    }

    #[test]
    fn rejects_empty_path() {
        let input = r#"
            [[engine]]
            id = "empty-path"
            path = ""
        "#;

        let config = EngineConfigFile::from_str(input).expect("parse config");
        let err = config.validate().expect_err("validate should fail");

        matches!(err, ConfigError::EmptyPath);
    }
}
