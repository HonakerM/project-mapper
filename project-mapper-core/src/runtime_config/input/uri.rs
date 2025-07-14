use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UriConfig {
    pub uri: String,
}
