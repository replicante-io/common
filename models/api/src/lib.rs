extern crate serde;
extern crate serde_derive;

use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Result of a component/dependency health check.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "details")]
pub enum HealthStatus {
    #[serde(rename = "HEALTHY")]
    Helathy,

    #[serde(rename = "DEGRADED")]
    Degraded(String),

    #[serde(rename = "FAILED")]
    Failed(String),
}
