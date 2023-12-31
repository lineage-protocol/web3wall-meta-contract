use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct DataStructFork {
    pub owner: String,
    pub cid: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenSeaAttributes {
    pub trait_type: String,
    pub value: String,
}
