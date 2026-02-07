use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EngineSpec {
    pub id: String,
    pub name: String,
    pub author: String,
    pub path: PathBuf,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}
