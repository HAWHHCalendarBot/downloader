use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EventEntry {
    pub name: String,
    pub location: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
}
