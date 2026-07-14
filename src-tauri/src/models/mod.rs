pub mod clip;
pub mod filter_rule;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub database_ready: bool,
    pub embedding_status: String,
    pub stage: String,
}
