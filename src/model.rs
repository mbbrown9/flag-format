#[derive(Debug, Clone, PartialEq)]
pub struct FlagFile {
    pub flags: Vec<Flag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
    pub rollout: Option<u8>,
    pub tags: Vec<String>,
}
