use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SetupAction {
    CreateDir { path: String },
    CreateFile { path: String, content: String },
    RemovePath { path: String },
    ResetWorld,
    // Future expansion: SetPermissions, CreateHidden, etc.
}
