use serde_derive::Deserialize;
use serde_derive::Serialize;

use super::ActionHistoryItem;
use super::ActionModel;

/// Action information returned by the API.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActionInfoResponse {
    pub action: ActionModel,
    pub history: Vec<ActionHistoryItem>,
}
