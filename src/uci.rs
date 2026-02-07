#[derive(Debug, Clone)]
pub struct UciEngineInfo {
    pub name: String,
    pub author: String,
}

#[derive(Debug)]
pub struct UciEngine {
    pub id: String,
    pub info: Option<UciEngineInfo>,
}
